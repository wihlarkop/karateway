use karateway_core::{
    models::{CreateRateLimitRequest, RateLimit, RateLimits, UpdateRateLimitRequest},
    KaratewayError, Result,
};
use sea_query::{Expr, Func, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct RateLimitRepository {
    pool: PgPool,
}

impl RateLimitRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateRateLimitRequest) -> Result<RateLimit> {
        let (sql, values) = Query::insert()
            .into_table(RateLimits::Table)
            .columns([
                RateLimits::Name,
                RateLimits::ApiRouteId,
                RateLimits::MaxRequests,
                RateLimits::WindowSeconds,
                RateLimits::IdentifierType,
                RateLimits::BurstSize,
            ])
            .values_panic([
                req.name.into(),
                req.api_route_id.into(),
                req.max_requests.into(),
                req.window_seconds.into(),
                req.identifier_type.to_string().into(),
                req.burst_size.into(),
            ])
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let limit = sqlx::query_as_with::<_, RateLimit, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(limit)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<RateLimit> {
        let (sql, values) = Query::select()
            .columns([
                RateLimits::Id,
                RateLimits::Name,
                RateLimits::ApiRouteId,
                RateLimits::MaxRequests,
                RateLimits::WindowSeconds,
                RateLimits::IdentifierType,
                RateLimits::IsActive,
                RateLimits::BurstSize,
                RateLimits::CreatedAt,
                RateLimits::UpdatedAt,
            ])
            .from(RateLimits::Table)
            .and_where(Expr::col(RateLimits::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let limit = sqlx::query_as_with::<_, RateLimit, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| KaratewayError::NotFound(format!("Rate limit with id {} not found", id)))?;

        Ok(limit)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<RateLimit>> {
        let offset = (page.saturating_sub(1)) * limit;

        let (sql, values) = Query::select()
            .columns([
                RateLimits::Id,
                RateLimits::Name,
                RateLimits::ApiRouteId,
                RateLimits::MaxRequests,
                RateLimits::WindowSeconds,
                RateLimits::IdentifierType,
                RateLimits::IsActive,
                RateLimits::BurstSize,
                RateLimits::CreatedAt,
                RateLimits::UpdatedAt,
            ])
            .from(RateLimits::Table)
            .order_by(RateLimits::CreatedAt, sea_query::Order::Desc)
            .limit(limit as u64)
            .offset(offset as u64)
            .build_sqlx(PostgresQueryBuilder);

        let limits = sqlx::query_as_with::<_, RateLimit, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(limits)
    }

    pub async fn count(&self) -> Result<u64> {
        let (sql, values) = Query::select()
            .expr(Func::count(Expr::col(RateLimits::Id)))
            .from(RateLimits::Table)
            .build_sqlx(PostgresQueryBuilder);

        let count: (i64,) = sqlx::query_as_with(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 as u64)
    }

    pub async fn list_by_route(&self, api_route_id: Uuid) -> Result<Vec<RateLimit>> {
        let (sql, values) = Query::select()
            .columns([
                RateLimits::Id,
                RateLimits::Name,
                RateLimits::ApiRouteId,
                RateLimits::MaxRequests,
                RateLimits::WindowSeconds,
                RateLimits::IdentifierType,
                RateLimits::IsActive,
                RateLimits::BurstSize,
                RateLimits::CreatedAt,
                RateLimits::UpdatedAt,
            ])
            .from(RateLimits::Table)
            .and_where(Expr::col(RateLimits::ApiRouteId).eq(api_route_id))
            .and_where(Expr::col(RateLimits::IsActive).eq(true))
            .order_by(RateLimits::CreatedAt, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let limits = sqlx::query_as_with::<_, RateLimit, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(limits)
    }

    pub async fn update(&self, id: Uuid, req: UpdateRateLimitRequest) -> Result<RateLimit> {
        let mut limit = self.find_by_id(id).await?;

        if let Some(name) = req.name {
            limit.name = name;
        }
        if let Some(api_route_id) = req.api_route_id {
            limit.api_route_id = Some(api_route_id);
        }
        if let Some(max_requests) = req.max_requests {
            limit.max_requests = max_requests;
        }
        if let Some(window_seconds) = req.window_seconds {
            limit.window_seconds = window_seconds;
        }
        if let Some(identifier_type) = req.identifier_type {
            limit.identifier_type = identifier_type;
        }
        if let Some(is_active) = req.is_active {
            limit.is_active = is_active;
        }
        if let Some(burst_size) = req.burst_size {
            limit.burst_size = Some(burst_size);
        }

        let (sql, values) = Query::update()
            .table(RateLimits::Table)
            .values([
                (RateLimits::Name, limit.name.clone().into()),
                (RateLimits::ApiRouteId, limit.api_route_id.into()),
                (RateLimits::MaxRequests, limit.max_requests.into()),
                (RateLimits::WindowSeconds, limit.window_seconds.into()),
                (RateLimits::IdentifierType, limit.identifier_type.to_string().into()),
                (RateLimits::IsActive, limit.is_active.into()),
                (RateLimits::BurstSize, limit.burst_size.into()),
            ])
            .and_where(Expr::col(RateLimits::Id).eq(id))
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let updated = sqlx::query_as_with::<_, RateLimit, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let (sql, values) = Query::delete()
            .from_table(RateLimits::Table)
            .and_where(Expr::col(RateLimits::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let result = sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(KaratewayError::NotFound(format!(
                "Rate limit with id {} not found",
                id
            )));
        }

        Ok(())
    }

    pub async fn list_active(&self) -> Result<Vec<RateLimit>> {
        let (sql, values) = Query::select()
            .columns([
                RateLimits::Id,
                RateLimits::Name,
                RateLimits::ApiRouteId,
                RateLimits::MaxRequests,
                RateLimits::WindowSeconds,
                RateLimits::IdentifierType,
                RateLimits::IsActive,
                RateLimits::BurstSize,
                RateLimits::CreatedAt,
                RateLimits::UpdatedAt,
            ])
            .from(RateLimits::Table)
            .and_where(Expr::col(RateLimits::IsActive).eq(true))
            .order_by(RateLimits::CreatedAt, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let limits = sqlx::query_as_with::<_, RateLimit, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(limits)
    }
}
