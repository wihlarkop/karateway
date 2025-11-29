use karateway_core::models::{AuditLog, AuditLogs};
use sea_query::{PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Audit logger service that handles async logging to database
#[derive(Clone)]
pub struct AuditLogger {
    tx: mpsc::UnboundedSender<AuditLog>,
}

impl AuditLogger {
    /// Create a new audit logger with a background worker
    pub fn new(pool: PgPool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn background worker to process audit logs
        tokio::spawn(audit_log_worker(pool, rx));

        Self { tx }
    }

    /// Log an audit event (non-blocking)
    pub fn log(&self, audit_log: AuditLog) {
        if let Err(e) = self.tx.send(audit_log) {
            error!("Failed to send audit log to worker: {}", e);
        }
    }
}

/// Background worker that processes audit logs and writes to database
async fn audit_log_worker(pool: PgPool, mut rx: mpsc::UnboundedReceiver<AuditLog>) {
    info!("Audit log worker started");

    while let Some(log) = rx.recv().await {
        if let Err(e) = save_audit_log(&pool, &log).await {
            error!(
                "Failed to save audit log to database: {} - Event: {:?}",
                e, log
            );
        }
    }

    info!("Audit log worker stopped");
}

/// Save an audit log entry to the database
async fn save_audit_log(pool: &PgPool, log: &AuditLog) -> Result<(), sqlx::Error> {
    let (sql, values) = Query::insert()
        .into_table(AuditLogs::Table)
        .columns([
            AuditLogs::Id,
            AuditLogs::EventType,
            AuditLogs::EventCategory,
            AuditLogs::Severity,
            AuditLogs::RequestMethod,
            AuditLogs::RequestPath,
            AuditLogs::ClientIp,
            AuditLogs::UserAgent,
            AuditLogs::ApiRouteId,
            AuditLogs::BackendServiceId,
            AuditLogs::Message,
            AuditLogs::Metadata,
            AuditLogs::StatusCode,
            AuditLogs::CreatedAt,
        ])
        .values_panic([
            log.id.into(),
            log.event_type.clone().into(),
            log.event_category.clone().into(),
            log.severity.clone().into(),
            log.request_method.clone().into(),
            log.request_path.clone().into(),
            log.client_ip.clone().into(),
            log.user_agent.clone().into(),
            log.api_route_id.into(),
            log.backend_service_id.into(),
            log.message.clone().into(),
            log.metadata.clone().into(),
            log.status_code.into(),
            log.created_at.into(),
        ])
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
