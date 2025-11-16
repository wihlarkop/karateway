use karateway_core::models::AuditLog;
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
    sqlx::query!(
        r#"
        INSERT INTO audit_logs (
            id, event_type, event_category, severity,
            request_method, request_path, client_ip, user_agent,
            api_route_id, backend_service_id, message, metadata, status_code,
            created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
        )
        "#,
        log.id,
        log.event_type,
        log.event_category,
        log.severity,
        log.request_method,
        log.request_path,
        log.client_ip,
        log.user_agent,
        log.api_route_id,
        log.backend_service_id,
        log.message,
        log.metadata,
        log.status_code,
        log.created_at
    )
    .execute(pool)
    .await?;

    Ok(())
}
