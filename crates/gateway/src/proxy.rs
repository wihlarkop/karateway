use async_trait::async_trait;
use bytes::Bytes;
use karateway_config::AuditLogger;
use karateway_core::models::{
    AuditEventCategory, AuditEventType, AuditLogBuilder, AuditSeverity, IdentifierType,
};
use pingora_core::upstreams::peer::{HttpPeer, Peer};
use pingora_core::Result;
use pingora_http::RequestHeader;
use pingora_proxy::{ProxyHttp, Session};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config_loader::ConfigLoader;
use crate::health_checker::HealthChecker;
use crate::rate_limiter::RateLimiter;
use crate::router::Router;
use crate::whitelist_validator::WhitelistValidator;

/// Karateway proxy context for each request
pub struct RequestContext {
    /// The upstream URL to proxy to
    pub upstream_host: String,
    pub upstream_port: u16,
    pub upstream_path: String,
    pub use_tls: bool,
    pub preserve_host: bool,
    pub route_id: Option<Uuid>,
}

/// Karateway proxy service
pub struct KaratewayProxy {
    router: Router,
    rate_limiter: Option<Arc<RateLimiter>>,
    health_checker: Arc<HealthChecker>,
    audit_logger: Arc<AuditLogger>,
}

impl KaratewayProxy {
    pub fn new(
        config_loader: Arc<ConfigLoader>,
        rate_limiter: Option<Arc<RateLimiter>>,
        health_checker: Arc<HealthChecker>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            router: Router::new(config_loader),
            rate_limiter,
            health_checker,
            audit_logger,
        }
    }

    /// Helper to extract client IP from session
    fn get_client_ip(session: &Session) -> Option<String> {
        session
            .req_header()
            .headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                session.client_addr().map(|addr| {
                    addr.as_inet()
                        .map(|inet| inet.ip().to_string())
                        .unwrap_or_else(|| addr.to_string())
                })
            })
    }

    /// Helper to extract user agent from session
    fn get_user_agent(session: &Session) -> Option<String> {
        session
            .req_header()
            .headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }
}

#[async_trait]
impl ProxyHttp for KaratewayProxy {
    type CTX = RequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RequestContext {
            upstream_host: String::new(),
            upstream_port: 80,
            upstream_path: String::new(),
            use_tls: false,
            preserve_host: false,
            route_id: None,
        }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        let req_header = session.req_header();
        let path = req_header.uri.path();
        let method = req_header.method.as_str();

        debug!("Incoming request: {} {}", method, path);

        // Find matching route and backend service
        let (route, service) = match self.router.route_request(path, method) {
            Some(result) => result,
            None => {
                warn!("No route found for {} {}", method, path);

                // Send 404 response
                let mut resp = pingora_http::ResponseHeader::build(404, None)?;
                resp.insert_header("Content-Length", "9")?;
                session.write_response_header(Box::new(resp), false).await?;
                session
                    .write_response_body(Some(b"Not Found".as_ref().into()), true)
                    .await?;

                return Ok(true); // Request handled
            }
        };

        // Store route ID in context
        ctx.route_id = Some(route.id);

        // Check whitelist rules
        if let Some(whitelist_rules) = self.router.get_whitelist_rules(&route.id) {
            debug!(
                "Whitelist rules are configured, checking {} rules for route {}",
                whitelist_rules.len(),
                route.id
            );

            // Get client IP for validation
            let client_ip = session
                .req_header()
                .headers
                .get("X-Forwarded-For")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
                .or_else(|| {
                    session.client_addr().map(|addr| {
                        addr.as_inet()
                            .map(|inet| inet.ip().to_string())
                            .unwrap_or_else(|| addr.to_string())
                    })
                });

            let (allowed, matching_rule) = WhitelistValidator::validate_request(
                &whitelist_rules,
                session.req_header(),
                client_ip.as_deref(),
            );

            if !allowed {
                warn!(
                    "Request denied by whitelist: route={}, path={}, method={}, client_ip={:?}",
                    route.path_pattern, path, method, client_ip
                );

                // Log audit event for whitelist denial
                let audit_log = AuditLogBuilder::new(
                    AuditEventType::WhitelistDenied,
                    AuditEventCategory::Whitelist,
                    AuditSeverity::Warning,
                    format!("Access denied by whitelist rules for {} {}", method, path),
                )
                .request_method(method)
                .request_path(path)
                .client_ip(client_ip.as_deref().unwrap_or("unknown"))
                .user_agent(Self::get_user_agent(session).unwrap_or_default())
                .api_route_id(route.id)
                .status_code(403)
                .build();

                self.audit_logger.log(audit_log);

                // Send 403 Forbidden response
                let mut resp = pingora_http::ResponseHeader::build(403, None)?;
                resp.insert_header("Content-Type", "application/json")?;

                let body = r#"{"error":"Forbidden","message":"Access denied by whitelist rules"}"#;
                let body_bytes = Bytes::from(body);

                resp.insert_header("Content-Length", &body_bytes.len().to_string())?;
                session.write_response_header(Box::new(resp), false).await?;
                session.write_response_body(Some(body_bytes), true).await?;

                return Ok(true); // Request handled
            }

            info!(
                "Request allowed by whitelist rule: {} (route={}, client_ip={:?})",
                matching_rule.unwrap_or_else(|| "no specific rule".to_string()),
                route.path_pattern,
                client_ip
            );
        } else {
            debug!("No whitelist rules configured for route {}", route.id);
        }

        // Check if backend service is healthy
        if !self.health_checker.is_healthy(&service.id) {
            warn!(
                "Backend service {} ({}) is unhealthy, returning 503",
                service.name, service.id
            );

            // Send 503 Service Unavailable response
            let mut resp = pingora_http::ResponseHeader::build(503, None)?;
            resp.insert_header("Content-Type", "application/json")?;
            let body = format!(
                r#"{{"error":"Service Unavailable","message":"Backend service {} is currently unhealthy"}}"#,
                service.name
            );
            let body_bytes = Bytes::from(body);

            resp.insert_header("Content-Length", &body_bytes.len().to_string())?;
            session.write_response_header(Box::new(resp), false).await?;
            session.write_response_body(Some(body_bytes), true).await?;

            return Ok(true); // Request handled
        }

        // Check rate limits if rate limiter is configured
        if let Some(rate_limiter) = &self.rate_limiter {
            debug!(
                "Rate limiter is configured, checking rate limits for route {}",
                route.id
            );
            if let Some(rate_limits) = self.router.get_rate_limits(&route.id) {
                debug!("Found {} rate limits to check", rate_limits.len());
                for limit in rate_limits {
                    debug!("Checking rate limit: {}", limit.name);
                    // Get identifier for rate limiting
                    let identifier = match limit.identifier_type {
                        IdentifierType::Ip => {
                            // Get client IP from headers or peer address
                            session
                                .req_header()
                                .headers
                                .get("X-Forwarded-For")
                                .and_then(|h| h.to_str().ok())
                                .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
                                .or_else(|| {
                                    // Extract just the IP address, not the port
                                    session.client_addr().map(|addr| {
                                        addr.as_inet()
                                            .map(|inet| inet.ip().to_string())
                                            .unwrap_or_else(|| addr.to_string())
                                    })
                                })
                                .unwrap_or_else(|| "unknown".to_string())
                        }
                        IdentifierType::ApiKey => {
                            // Get API key from header
                            session
                                .req_header()
                                .headers
                                .get("X-API-Key")
                                .and_then(|h| h.to_str().ok())
                                .unwrap_or("no-api-key")
                                .to_string()
                        }
                        IdentifierType::UserId => {
                            // Get user ID from header (JWT, session, etc.)
                            session
                                .req_header()
                                .headers
                                .get("X-User-ID")
                                .and_then(|h| h.to_str().ok())
                                .unwrap_or("no-user-id")
                                .to_string()
                        }
                        IdentifierType::Global => {
                            // Global rate limit for all requests
                            "global".to_string()
                        }
                    };

                    let rate_limit_key =
                        format!("{}:{}:{}", route.id, limit.identifier_type, identifier);

                    // Check rate limit
                    let (allowed, remaining, reset_time) = if let Some(burst) = limit.burst_size {
                        rate_limiter
                            .check_rate_limit_with_burst(
                                &rate_limit_key,
                                limit.max_requests,
                                limit.window_seconds,
                                burst,
                            )
                            .await
                    } else {
                        rate_limiter
                            .check_rate_limit(
                                &rate_limit_key,
                                limit.max_requests,
                                limit.window_seconds,
                            )
                            .await
                    }
                    .map_err(|e| {
                        warn!("Rate limiter error: {}", e);
                        pingora_core::Error::because(
                            pingora_core::ErrorType::InternalError,
                            "Rate limiter error",
                            e,
                        )
                    })?;

                    if !allowed {
                        info!(
                            "Rate limit exceeded: route={}, identifier_type={}, identifier={}, limit={}",
                            route.path_pattern, limit.identifier_type, identifier, limit.name
                        );

                        // Log audit event for rate limit exceeded
                        let mut metadata = serde_json::Map::new();
                        metadata.insert(
                            "limit_name".to_string(),
                            serde_json::Value::String(limit.name.clone()),
                        );
                        metadata.insert(
                            "identifier_type".to_string(),
                            serde_json::Value::String(limit.identifier_type.to_string()),
                        );
                        metadata.insert(
                            "identifier".to_string(),
                            serde_json::Value::String(identifier.clone()),
                        );
                        metadata.insert(
                            "max_requests".to_string(),
                            serde_json::Value::Number(limit.max_requests.into()),
                        );
                        metadata.insert(
                            "window_seconds".to_string(),
                            serde_json::Value::Number(limit.window_seconds.into()),
                        );

                        let audit_log = AuditLogBuilder::new(
                            AuditEventType::RateLimitExceeded,
                            AuditEventCategory::RateLimit,
                            AuditSeverity::Warning,
                            format!(
                                "Rate limit '{}' exceeded for {} {} (identifier: {})",
                                limit.name, method, path, identifier
                            ),
                        )
                        .request_method(method)
                        .request_path(path)
                        .client_ip(Self::get_client_ip(session).unwrap_or_default())
                        .user_agent(Self::get_user_agent(session).unwrap_or_default())
                        .api_route_id(route.id)
                        .metadata(serde_json::Value::Object(metadata))
                        .status_code(429)
                        .build();

                        self.audit_logger.log(audit_log);

                        // Rate limit exceeded - return 429
                        let mut resp = pingora_http::ResponseHeader::build(429, None)?;
                        resp.insert_header("Content-Type", "application/json")?;
                        resp.insert_header("X-RateLimit-Limit", &limit.max_requests.to_string())?;
                        resp.insert_header("X-RateLimit-Remaining", "0")?;
                        resp.insert_header("X-RateLimit-Reset", &reset_time.to_string())?;
                        resp.insert_header("Retry-After", &limit.window_seconds.to_string())?;

                        let body = format!(
                            r#"{{"error":"Rate limit exceeded","retry_after":{},"limit":"{}"}}"#,
                            limit.window_seconds, limit.name
                        );
                        let body_bytes = Bytes::from(body);

                        resp.insert_header("Content-Length", &body_bytes.len().to_string())?;
                        session.write_response_header(Box::new(resp), false).await?;
                        session.write_response_body(Some(body_bytes), true).await?;

                        return Ok(true); // Request handled
                    } else {
                        // Add rate limit headers to response (will be added in response_filter)
                        debug!(
                            "Rate limit check passed: remaining={}, reset_in={}s",
                            remaining,
                            reset_time.saturating_sub(
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            )
                        );
                    }
                }
            }
        }

        // Parse backend URL
        let backend_url = url::Url::parse(&service.base_url).map_err(|e| {
            pingora_core::Error::because(
                pingora_core::ErrorType::InternalError,
                format!("Invalid backend URL: {}", e),
                e,
            )
        })?;

        // Transform path if needed
        let transformed_path = self.router.transform_path(&route, path);

        // Build query string
        let query = req_header
            .uri
            .query()
            .map(|q| format!("?{}", q))
            .unwrap_or_default();
        let full_path = format!("{}{}", transformed_path, query);

        // Store upstream information in context
        ctx.upstream_host = backend_url.host_str().unwrap_or("localhost").to_string();
        ctx.upstream_port = backend_url
            .port()
            .unwrap_or(if backend_url.scheme() == "https" {
                443
            } else {
                80
            });
        ctx.upstream_path = full_path;
        ctx.use_tls = backend_url.scheme() == "https";
        ctx.preserve_host = route.preserve_host_header;

        debug!(
            "Route config: preserve_host_header={}, route_id={}",
            route.preserve_host_header, route.id
        );

        debug!(
            "Routing to {}:{}{} (TLS: {})",
            ctx.upstream_host, ctx.upstream_port, ctx.upstream_path, ctx.use_tls
        );

        Ok(false) // Continue to upstream
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let mut peer = HttpPeer::new(
            (&ctx.upstream_host as &str, ctx.upstream_port),
            ctx.use_tls,
            ctx.upstream_host.clone(),
        );

        // Configure TLS options for HTTPS backends
        if ctx.use_tls {
            if let Some(options) = peer.get_mut_peer_options() {
                // Temporarily disable cert verification to test connection
                // TODO: Re-enable with proper certificate configuration
                options.verify_cert = false;
                options.verify_hostname = false;
            }
        }

        debug!(
            "Created upstream peer: {}:{} (TLS: {})",
            ctx.upstream_host, ctx.upstream_port, ctx.use_tls
        );

        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Update the request URI with the transformed path
        upstream_request.set_uri(ctx.upstream_path.parse().map_err(|e| {
            pingora_core::Error::because(
                pingora_core::ErrorType::InternalError,
                format!("Invalid URI: {}", e),
                e,
            )
        })?);

        // Update Host header if not preserving original
        if !ctx.preserve_host {
            debug!(
                "Updating Host header from {:?} to {}",
                upstream_request.headers.get("host"),
                ctx.upstream_host
            );
            upstream_request
                .insert_header("Host", &ctx.upstream_host)
                .map_err(|e| {
                    pingora_core::Error::because(
                        pingora_core::ErrorType::InternalError,
                        "Failed to insert Host header",
                        e,
                    )
                })?;
        } else {
            debug!(
                "Preserving original Host header: {:?}",
                upstream_request.headers.get("host")
            );
        }

        // Add X-Forwarded headers
        upstream_request
            .insert_header(
                "X-Forwarded-Proto",
                if ctx.use_tls { "https" } else { "http" },
            )
            .ok();

        debug!(
            "Upstream request: {} {} with Host: {:?}",
            upstream_request.method,
            upstream_request.uri,
            upstream_request.headers.get("host")
        );

        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut pingora_http::ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Add custom response headers
        upstream_response
            .insert_header("X-Powered-By", "Karateway")
            .ok();

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _error: Option<&pingora_core::Error>,
        ctx: &mut Self::CTX,
    ) {
        let req_header = session.req_header();
        let status = session
            .response_written()
            .map(|r| r.status.as_u16())
            .unwrap_or(0);

        info!(
            method = %req_header.method,
            path = %req_header.uri.path(),
            status = status,
            upstream = format!("{}:{}{}", ctx.upstream_host, ctx.upstream_port, ctx.upstream_path),
            "Request completed"
        );
    }
}
