use karateway_core::{
    models::{AuditLog, AuditLogs},
    Result,
};
use sea_query::{Expr, Func, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AuditLogRepository {
    pool: PgPool,
}

impl AuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        let (sql, values) = Query::select()
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
            .from(AuditLogs::Table)
            .order_by(AuditLogs::CreatedAt, sea_query::Order::Desc)
            .limit(limit as u64)
            .offset(offset as u64)
            .build_sqlx(PostgresQueryBuilder);

        let logs = sqlx::query_as_with::<_, AuditLog, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(logs)
    }

    pub async fn count(&self) -> Result<i64> {
        let (sql, values) = Query::select()
            .expr(Func::count(Expr::col(AuditLogs::Id)))
            .from(AuditLogs::Table)
            .build_sqlx(PostgresQueryBuilder);

        let count: (i64,) = sqlx::query_as_with(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}
