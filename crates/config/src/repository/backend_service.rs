use karateway_core::{
    models::{BackendService, CreateBackendServiceRequest, UpdateBackendServiceRequest},
    KaratewayError, Result,
};
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
        let service = sqlx::query_as::<_, BackendService>(
            r#"
            INSERT INTO backend_services (
                name, description, base_url, health_check_url,
                health_check_interval_seconds, timeout_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.base_url)
        .bind(&req.health_check_url)
        .bind(req.health_check_interval_seconds)
        .bind(req.timeout_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(service)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<BackendService> {
        let service = sqlx::query_as::<_, BackendService>(
            r#"
            SELECT * FROM backend_services WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            KaratewayError::NotFound(format!("Backend service with id {} not found", id))
        })?;

        Ok(service)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<BackendService>> {
        let service = sqlx::query_as::<_, BackendService>(
            r#"
            SELECT * FROM backend_services WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(service)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<BackendService>> {
        let offset = (page.saturating_sub(1)) * limit;

        let services = sqlx::query_as::<_, BackendService>(
            r#"
            SELECT * FROM backend_services
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(services)
    }

    pub async fn count(&self) -> Result<u64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM backend_services
            "#,
        )
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
        let updated = sqlx::query_as::<_, BackendService>(
            r#"
            UPDATE backend_services
            SET name = $1, description = $2, base_url = $3,
                health_check_url = $4, health_check_interval_seconds = $5,
                timeout_ms = $6, is_active = $7, updated_at = CURRENT_TIMESTAMP
            WHERE id = $8
            RETURNING *
            "#,
        )
        .bind(&service.name)
        .bind(&service.description)
        .bind(&service.base_url)
        .bind(&service.health_check_url)
        .bind(service.health_check_interval_seconds)
        .bind(service.timeout_ms)
        .bind(service.is_active)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM backend_services WHERE id = $1
            "#,
        )
        .bind(id)
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
        let services = sqlx::query_as::<_, BackendService>(
            r#"
            SELECT * FROM backend_services
            WHERE is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(services)
    }
}
