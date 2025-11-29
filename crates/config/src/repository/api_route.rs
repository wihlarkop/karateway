use karateway_core::{
    models::{ApiRoute, ApiRoutes, CreateApiRouteRequest, UpdateApiRouteRequest},
    KaratewayError, Result,
};
use sea_query::{Expr, Func, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
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
        let (sql, values) = Query::insert()
            .into_table(ApiRoutes::Table)
            .columns([
                ApiRoutes::PathPattern,
                ApiRoutes::Method,
                ApiRoutes::BackendServiceId,
                ApiRoutes::StripPathPrefix,
                ApiRoutes::PreserveHostHeader,
                ApiRoutes::TimeoutMs,
                ApiRoutes::Priority,
                ApiRoutes::Metadata,
            ])
            .values_panic([
                req.path_pattern.into(),
                req.method.to_string().into(),
                req.backend_service_id.into(),
                req.strip_path_prefix.unwrap_or(false).into(),
                req.preserve_host_header.unwrap_or(false).into(),
                req.timeout_ms.into(),
                req.priority.unwrap_or(0).into(),
                req.metadata.unwrap_or(serde_json::json!({})).into(),
            ])
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let route = sqlx::query_as_with::<_, ApiRoute, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(route)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<ApiRoute> {
        let (sql, values) = Query::select()
            .columns([
                ApiRoutes::Id,
                ApiRoutes::PathPattern,
                ApiRoutes::Method,
                ApiRoutes::BackendServiceId,
                ApiRoutes::StripPathPrefix,
                ApiRoutes::PreserveHostHeader,
                ApiRoutes::TimeoutMs,
                ApiRoutes::IsActive,
                ApiRoutes::Priority,
                ApiRoutes::Metadata,
                ApiRoutes::CreatedAt,
                ApiRoutes::UpdatedAt,
            ])
            .from(ApiRoutes::Table)
            .and_where(Expr::col(ApiRoutes::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let route = sqlx::query_as_with::<_, ApiRoute, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| {
                KaratewayError::NotFound(format!("API route with id {} not found", id))
            })?;

        Ok(route)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<ApiRoute>> {
        let offset = (page.saturating_sub(1)) * limit;

        let (sql, values) = Query::select()
            .columns([
                ApiRoutes::Id,
                ApiRoutes::PathPattern,
                ApiRoutes::Method,
                ApiRoutes::BackendServiceId,
                ApiRoutes::StripPathPrefix,
                ApiRoutes::PreserveHostHeader,
                ApiRoutes::TimeoutMs,
                ApiRoutes::IsActive,
                ApiRoutes::Priority,
                ApiRoutes::Metadata,
                ApiRoutes::CreatedAt,
                ApiRoutes::UpdatedAt,
            ])
            .from(ApiRoutes::Table)
            .order_by(ApiRoutes::Priority, sea_query::Order::Desc)
            .order_by(ApiRoutes::CreatedAt, sea_query::Order::Desc)
            .limit(limit as u64)
            .offset(offset as u64)
            .build_sqlx(PostgresQueryBuilder);

        let routes = sqlx::query_as_with::<_, ApiRoute, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(routes)
    }

    pub async fn count(&self) -> Result<u64> {
        let (sql, values) = Query::select()
            .expr(Func::count(Expr::col(ApiRoutes::Id)))
            .from(ApiRoutes::Table)
            .build_sqlx(PostgresQueryBuilder);

        let count: (i64,) = sqlx::query_as_with(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 as u64)
    }

    pub async fn list_by_backend_service(&self, backend_service_id: Uuid) -> Result<Vec<ApiRoute>> {
        let (sql, values) = Query::select()
            .columns([
                ApiRoutes::Id,
                ApiRoutes::PathPattern,
                ApiRoutes::Method,
                ApiRoutes::BackendServiceId,
                ApiRoutes::StripPathPrefix,
                ApiRoutes::PreserveHostHeader,
                ApiRoutes::TimeoutMs,
                ApiRoutes::IsActive,
                ApiRoutes::Priority,
                ApiRoutes::Metadata,
                ApiRoutes::CreatedAt,
                ApiRoutes::UpdatedAt,
            ])
            .from(ApiRoutes::Table)
            .and_where(Expr::col(ApiRoutes::BackendServiceId).eq(backend_service_id))
            .order_by(ApiRoutes::Priority, sea_query::Order::Desc)
            .order_by(ApiRoutes::CreatedAt, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let routes = sqlx::query_as_with::<_, ApiRoute, _>(&sql, values)
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
        let (sql, values) = Query::update()
            .table(ApiRoutes::Table)
            .values([
                (ApiRoutes::PathPattern, route.path_pattern.clone().into()),
                (ApiRoutes::Method, route.method.to_string().into()),
                (ApiRoutes::BackendServiceId, route.backend_service_id.into()),
                (ApiRoutes::StripPathPrefix, route.strip_path_prefix.into()),
                (
                    ApiRoutes::PreserveHostHeader,
                    route.preserve_host_header.into(),
                ),
                (ApiRoutes::TimeoutMs, route.timeout_ms.into()),
                (ApiRoutes::IsActive, route.is_active.into()),
                (ApiRoutes::Priority, route.priority.into()),
                (ApiRoutes::Metadata, route.metadata.clone().into()),
            ])
            .and_where(Expr::col(ApiRoutes::Id).eq(id))
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let updated = sqlx::query_as_with::<_, ApiRoute, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let (sql, values) = Query::delete()
            .from_table(ApiRoutes::Table)
            .and_where(Expr::col(ApiRoutes::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let result = sqlx::query_with(&sql, values).execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(KaratewayError::NotFound(format!(
                "API route with id {} not found",
                id
            )));
        }

        Ok(())
    }

    pub async fn list_active(&self) -> Result<Vec<ApiRoute>> {
        let (sql, values) = Query::select()
            .columns([
                ApiRoutes::Id,
                ApiRoutes::PathPattern,
                ApiRoutes::Method,
                ApiRoutes::BackendServiceId,
                ApiRoutes::StripPathPrefix,
                ApiRoutes::PreserveHostHeader,
                ApiRoutes::TimeoutMs,
                ApiRoutes::IsActive,
                ApiRoutes::Priority,
                ApiRoutes::Metadata,
                ApiRoutes::CreatedAt,
                ApiRoutes::UpdatedAt,
            ])
            .from(ApiRoutes::Table)
            .and_where(Expr::col(ApiRoutes::IsActive).eq(true))
            .order_by(ApiRoutes::Priority, sea_query::Order::Desc)
            .order_by(ApiRoutes::CreatedAt, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let routes = sqlx::query_as_with::<_, ApiRoute, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(routes)
    }
}
