use karateway_core::{
    models::{ApiRoute, CreateApiRouteRequest, UpdateApiRouteRequest},
    KaratewayError, Result,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiRouteRepository {
    pool: PgPool,
}

impl ApiRouteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateApiRouteRequest) -> Result<ApiRoute> {
        let route = sqlx::query_as::<_, ApiRoute>(
            r#"
            INSERT INTO api_routes (
                path_pattern, method, backend_service_id, strip_path_prefix,
                preserve_host_header, timeout_ms, priority, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&req.path_pattern)
        .bind(&req.method)
        .bind(req.backend_service_id)
        .bind(req.strip_path_prefix.unwrap_or(false))
        .bind(req.preserve_host_header.unwrap_or(false))
        .bind(req.timeout_ms)
        .bind(req.priority.unwrap_or(0))
        .bind(req.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(route)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<ApiRoute> {
        let route = sqlx::query_as::<_, ApiRoute>(
            r#"
            SELECT * FROM api_routes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| KaratewayError::NotFound(format!("API route with id {} not found", id)))?;

        Ok(route)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<ApiRoute>> {
        let offset = (page.saturating_sub(1)) * limit;

        let routes = sqlx::query_as::<_, ApiRoute>(
            r#"
            SELECT * FROM api_routes
            ORDER BY priority DESC, created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(routes)
    }

    pub async fn count(&self) -> Result<u64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM api_routes
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 as u64)
    }

    pub async fn list_by_backend_service(&self, backend_service_id: Uuid) -> Result<Vec<ApiRoute>> {
        let routes = sqlx::query_as::<_, ApiRoute>(
            r#"
            SELECT * FROM api_routes
            WHERE backend_service_id = $1
            ORDER BY priority DESC, created_at DESC
            "#,
        )
        .bind(backend_service_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(routes)
    }

    pub async fn update(&self, id: Uuid, req: UpdateApiRouteRequest) -> Result<ApiRoute> {
        let mut route = self.find_by_id(id).await?;

        // Apply updates
        if let Some(path_pattern) = req.path_pattern {
            route.path_pattern = path_pattern;
        }
        if let Some(method) = req.method {
            route.method = method;
        }
        if let Some(backend_service_id) = req.backend_service_id {
            route.backend_service_id = backend_service_id;
        }
        if let Some(strip_path_prefix) = req.strip_path_prefix {
            route.strip_path_prefix = strip_path_prefix;
        }
        if let Some(preserve_host_header) = req.preserve_host_header {
            route.preserve_host_header = preserve_host_header;
        }
        if let Some(timeout_ms) = req.timeout_ms {
            route.timeout_ms = Some(timeout_ms);
        }
        if let Some(is_active) = req.is_active {
            route.is_active = is_active;
        }
        if let Some(priority) = req.priority {
            route.priority = priority;
        }
        if let Some(metadata) = req.metadata {
            route.metadata = metadata;
        }

        // Save to database
        let updated = sqlx::query_as::<_, ApiRoute>(
            r#"
            UPDATE api_routes
            SET path_pattern = $1, method = $2, backend_service_id = $3,
                strip_path_prefix = $4, preserve_host_header = $5, timeout_ms = $6,
                is_active = $7, priority = $8, metadata = $9, updated_at = CURRENT_TIMESTAMP
            WHERE id = $10
            RETURNING *
            "#,
        )
        .bind(&route.path_pattern)
        .bind(&route.method)
        .bind(route.backend_service_id)
        .bind(route.strip_path_prefix)
        .bind(route.preserve_host_header)
        .bind(route.timeout_ms)
        .bind(route.is_active)
        .bind(route.priority)
        .bind(&route.metadata)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM api_routes WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(KaratewayError::NotFound(format!(
                "API route with id {} not found",
                id
            )));
        }

        Ok(())
    }

    pub async fn list_active(&self) -> Result<Vec<ApiRoute>> {
        let routes = sqlx::query_as::<_, ApiRoute>(
            r#"
            SELECT * FROM api_routes
            WHERE is_active = true
            ORDER BY priority DESC, created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(routes)
    }
}
