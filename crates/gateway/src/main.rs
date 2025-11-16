mod config_loader;
mod health_checker;
mod proxy;
mod rate_limiter;
mod router;
mod whitelist_validator;

use anyhow::Result;
use pingora_core::server::Server;
use pingora_proxy::http_proxy_service;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config_loader::ConfigLoader;
use health_checker::HealthChecker;
use proxy::KaratewayProxy;
use rate_limiter::RateLimiter;

fn main() -> Result<()> {
    // Initialize environment variables
    karateway_config::init_env();

    // Initialize rustls crypto provider (required for rustls TLS)
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize tracing
    init_tracing();

    info!("Starting Karateway Gateway v{}", env!("CARGO_PKG_VERSION"));

    // Load application configuration synchronously using blocking runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (config_loader, audit_logger) = rt.block_on(async {
        // Load application configuration
        let app_config = karateway_config::AppConfig::from_env()?;
        info!("Loaded configuration from environment");

        // Initialize database connection pool
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&app_config.database_url)
            .await?;
        info!("Connected to PostgreSQL database");

        // Initialize audit logger
        let audit_logger = Arc::new(karateway_config::AuditLogger::new(db_pool.clone()));
        info!("Audit logger initialized");

        // Initialize configuration loader
        let config_loader = Arc::new(ConfigLoader::new(db_pool.clone()));

        // Load initial configuration
        config_loader.load_config().await?;
        info!("Loaded initial configuration from database");

        Ok::<_, anyhow::Error>((config_loader, audit_logger))
    })?;

    // Start configuration reload background task on the runtime
    let config_loader_clone = config_loader.clone();
    rt.spawn(async move {
        config_loader_clone.start_reload_watcher().await;
    });
    info!("Started configuration reload watcher");

    // Initialize rate limiter (optional - only if Redis is configured)
    let rate_limiter = rt.block_on(async {
        let app_config = karateway_config::AppConfig::from_env().ok()?;
        match RateLimiter::new(&app_config.redis_url) {
            Ok(limiter) => {
                info!("Rate limiter initialized with Redis");
                Some(Arc::new(limiter))
            }
            Err(e) => {
                info!("Rate limiter not initialized (Redis not available): {}", e);
                None
            }
        }
    });

    // Initialize health checker and start background task on the runtime
    let health_checker = Arc::new(HealthChecker::new(config_loader.clone()));
    let health_checker_clone = health_checker.clone();
    rt.spawn(async move {
        health_checker_clone.start_background_checker();
    });
    info!("Health checker started");

    // Create Pingora server
    let mut server = Server::new(None)?;
    server.bootstrap();

    // Create proxy service with rate limiter, health checker, and audit logger
    let proxy = KaratewayProxy::new(config_loader, rate_limiter, health_checker, audit_logger);
    let mut proxy_service = http_proxy_service(&server.configuration, proxy);

    // Add TCP listener for HTTP
    proxy_service.add_tcp("0.0.0.0:8080");
    info!("Gateway server listening on 0.0.0.0:8080 (HTTP)");

    // Try to add TLS listener if certificate exists
    let cert_path = "certs/cert.pem";
    let key_path = "certs/key.pem";

    if std::path::Path::new(cert_path).exists() && std::path::Path::new(key_path).exists() {
        match pingora_core::listeners::tls::TlsSettings::intermediate(cert_path, key_path) {
            Ok(mut tls_settings) => {
                tls_settings.enable_h2();
                proxy_service.add_tls_with_settings("0.0.0.0:8443", None, tls_settings);
                info!("Gateway server listening on 0.0.0.0:8443 (HTTPS)");
            }
            Err(e) => {
                info!("TLS not configured: {} (cert/key not found or invalid)", e);
            }
        }
    } else {
        info!(
            "TLS not configured: certificate files not found at {}/{}",
            cert_path, key_path
        );
        info!("To enable HTTPS, generate certificates: openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem -days 365 -nodes -subj \"/CN=localhost\"");
    }

    // Add service to server
    server.add_service(proxy_service);

    // Keep runtime alive by moving it into a thread
    std::thread::spawn(move || {
        rt.block_on(async {
            // Keep runtime alive forever
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        })
    });

    // Run the server (blocks forever)
    server.run_forever();
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,karateway_gateway=debug,pingora_core=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();
}
