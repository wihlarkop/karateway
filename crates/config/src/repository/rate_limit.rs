use karateway_core::{
    models::{CreateRateLimitRequest, RateLimit, UpdateRateLimitRequest},
    KaratewayError, Result,
};
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
        let limit = sqlx::query_as::<_, RateLimit>(
            r#"
            INSERT INTO rate_limits (
                name, api_route_id, max_requests, window_seconds,
                identifier_type, burst_size
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(req.api_route_id)
        .bind(req.max_requests)
        .bind(req.window_seconds)
        .bind(&req.identifier_type)
        .bind(req.burst_size)
        .fetch_one(&self.pool)
        .await?;

        Ok(limit)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<RateLimit> {
        let limit = sqlx::query_as::<_, RateLimit>(
            r#"
            SELECT * FROM rate_limits WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| KaratewayError::NotFound(format!("Rate limit with id {} not found", id)))?;

        Ok(limit)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<RateLimit>> {
        let offset = (page.saturating_sub(1)) * limit;

        let limits = sqlx::query_as::<_, RateLimit>(
            r#"
            SELECT * FROM rate_limits
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(limits)
    }

    pub async fn count(&self) -> Result<u64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM rate_limits
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 as u64)
    }

    pub async fn list_by_route(&self, api_route_id: Uuid) -> Result<Vec<RateLimit>> {
        let limits = sqlx::query_as::<_, RateLimit>(
            r#"
            SELECT * FROM rate_limits
            WHERE api_route_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(api_route_id)
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

        let updated = sqlx::query_as::<_, RateLimit>(
            r#"
            UPDATE rate_limits
            SET name = $1, api_route_id = $2, max_requests = $3,
                window_seconds = $4, identifier_type = $5, is_active = $6,
                burst_size = $7, updated_at = CURRENT_TIMESTAMP
            WHERE id = $8
            RETURNING *
            "#,
        )
        .bind(&limit.name)
        .bind(limit.api_route_id)
        .bind(limit.max_requests)
        .bind(limit.window_seconds)
        .bind(&limit.identifier_type)
        .bind(limit.is_active)
        .bind(limit.burst_size)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM rate_limits WHERE id = $1
            "#,
        )
        .bind(id)
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
        let limits = sqlx::query_as::<_, RateLimit>(
            r#"
            SELECT * FROM rate_limits
            WHERE is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(limits)
    }
}
