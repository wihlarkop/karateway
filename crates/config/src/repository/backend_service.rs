use karateway_core::{
    models::{BackendService, BackendServices, CreateBackendServiceRequest, UpdateBackendServiceRequest},
    KaratewayError, Result,
};
use sea_query::{Expr, Func, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct BackendServiceRepository {
    pool: PgPool,
}

impl BackendServiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateBackendServiceRequest) -> Result<BackendService> {
        let (sql, values) = Query::insert()
            .into_table(BackendServices::Table)
            .columns([
                BackendServices::Name,
                BackendServices::Description,
                BackendServices::BaseUrl,
                BackendServices::HealthCheckUrl,
                BackendServices::HealthCheckIntervalSeconds,
                BackendServices::TimeoutMs,
            ])
            .values_panic([
                req.name.into(),
                req.description.into(),
                req.base_url.into(),
                req.health_check_url.into(),
                req.health_check_interval_seconds.into(),
                req.timeout_ms.into(),
            ])
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let service = sqlx::query_as_with::<_, BackendService, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(service)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<BackendService> {
        let (sql, values) = Query::select()
            .columns([
                BackendServices::Id,
                BackendServices::Name,
                BackendServices::Description,
                BackendServices::BaseUrl,
                BackendServices::HealthCheckUrl,
                BackendServices::HealthCheckIntervalSeconds,
                BackendServices::TimeoutMs,
                BackendServices::IsActive,
                BackendServices::CreatedAt,
                BackendServices::UpdatedAt,
            ])
            .from(BackendServices::Table)
            .and_where(Expr::col(BackendServices::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let service = sqlx::query_as_with::<_, BackendService, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| {
                KaratewayError::NotFound(format!("Backend service with id {} not found", id))
            })?;

        Ok(service)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<BackendService>> {
        let (sql, values) = Query::select()
            .columns([
                BackendServices::Id,
                BackendServices::Name,
                BackendServices::Description,
                BackendServices::BaseUrl,
                BackendServices::HealthCheckUrl,
                BackendServices::HealthCheckIntervalSeconds,
                BackendServices::TimeoutMs,
                BackendServices::IsActive,
                BackendServices::CreatedAt,
                BackendServices::UpdatedAt,
            ])
            .from(BackendServices::Table)
            .and_where(Expr::col(BackendServices::Name).eq(name))
            .build_sqlx(PostgresQueryBuilder);

        let service = sqlx::query_as_with::<_, BackendService, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(service)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<BackendService>> {
        let offset = (page.saturating_sub(1)) * limit;

        let (sql, values) = Query::select()
            .columns([
                BackendServices::Id,
                BackendServices::Name,
                BackendServices::Description,
                BackendServices::BaseUrl,
                BackendServices::HealthCheckUrl,
                BackendServices::HealthCheckIntervalSeconds,
                BackendServices::TimeoutMs,
                BackendServices::IsActive,
                BackendServices::CreatedAt,
                BackendServices::UpdatedAt,
            ])
            .from(BackendServices::Table)
            .order_by(BackendServices::CreatedAt, sea_query::Order::Desc)
            .limit(limit as u64)
            .offset(offset as u64)
            .build_sqlx(PostgresQueryBuilder);

        let services = sqlx::query_as_with::<_, BackendService, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(services)
    }

    pub async fn count(&self) -> Result<u64> {
        let (sql, values) = Query::select()
            .expr(Func::count(Expr::col(BackendServices::Id)))
            .from(BackendServices::Table)
            .build_sqlx(PostgresQueryBuilder);

        let count: (i64,) = sqlx::query_as_with(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 as u64)
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateBackendServiceRequest,
    ) -> Result<BackendService> {
        // Fetch existing service
        let mut service = self.find_by_id(id).await?;

        // Apply updates
        if let Some(name) = req.name {
            service.name = name;
        }
        if let Some(description) = req.description {
            service.description = Some(description);
        }
        if let Some(base_url) = req.base_url {
            service.base_url = base_url;
        }
        if let Some(health_check_url) = req.health_check_url {
            service.health_check_url = Some(health_check_url);
        }
        if let Some(interval) = req.health_check_interval_seconds {
            service.health_check_interval_seconds = Some(interval);
        }
        if let Some(timeout) = req.timeout_ms {
            service.timeout_ms = Some(timeout);
        }
        if let Some(is_active) = req.is_active {
            service.is_active = is_active;
        }

        // Save to database
        let (sql, values) = Query::update()
            .table(BackendServices::Table)
            .values([
                (BackendServices::Name, service.name.clone().into()),
                (BackendServices::Description, service.description.clone().into()),
                (BackendServices::BaseUrl, service.base_url.clone().into()),
                (BackendServices::HealthCheckUrl, service.health_check_url.clone().into()),
                (BackendServices::HealthCheckIntervalSeconds, service.health_check_interval_seconds.into()),
                (BackendServices::TimeoutMs, service.timeout_ms.into()),
                (BackendServices::IsActive, service.is_active.into()),
            ])
            .and_where(Expr::col(BackendServices::Id).eq(id))
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let updated = sqlx::query_as_with::<_, BackendService, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let (sql, values) = Query::delete()
            .from_table(BackendServices::Table)
            .and_where(Expr::col(BackendServices::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let result = sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(KaratewayError::NotFound(format!(
                "Backend service with id {} not found",
                id
            )));
        }

        Ok(())
    }

    pub async fn list_active(&self) -> Result<Vec<BackendService>> {
        let (sql, values) = Query::select()
            .columns([
                BackendServices::Id,
                BackendServices::Name,
                BackendServices::Description,
                BackendServices::BaseUrl,
                BackendServices::HealthCheckUrl,
                BackendServices::HealthCheckIntervalSeconds,
                BackendServices::TimeoutMs,
                BackendServices::IsActive,
                BackendServices::CreatedAt,
                BackendServices::UpdatedAt,
            ])
            .from(BackendServices::Table)
            .and_where(Expr::col(BackendServices::IsActive).eq(true))
            .order_by(BackendServices::CreatedAt, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let services = sqlx::query_as_with::<_, BackendService, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(services)
    }
}
