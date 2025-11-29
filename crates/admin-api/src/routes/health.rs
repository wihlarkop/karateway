use axum::{extract::State, Json};
use karateway_core::JsonResponse;
use redis::AsyncCommands;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseStatus {
    pub connected: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedisStatus {
    pub connected: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: DatabaseStatus,
    pub redis: RedisStatus,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = JsonResponse<HealthResponse>)
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<AppState>) -> Json<JsonResponse<HealthResponse>> {
    // Check database connection
    let (sql, values) = Query::select()
        .expr(Expr::value(1))
        .build_sqlx(PostgresQueryBuilder);

    let database = match sqlx::query_with(&sql, values)
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(_) => DatabaseStatus {
            connected: true,
            message: "Database connection healthy".to_string(),
        },
        Err(e) => DatabaseStatus {
            connected: false,
            message: format!("Database connection failed: {}", e),
        },
    };

    // Check Redis connection
    let redis = match state.redis_pool.get().await {
        Ok(mut conn) => {
            // Try to ping Redis
            match conn.set::<&str, &str, String>("health_check", "ok").await {
                Ok(_) => RedisStatus {
                    connected: true,
                    message: "Redis connection healthy".to_string(),
                },
                Err(e) => RedisStatus {
                    connected: false,
                    message: format!("Redis ping failed: {}", e),
                },
            }
        }
        Err(e) => RedisStatus {
            connected: false,
            message: format!("Redis connection failed: {}", e),
        },
    };

    let overall_status = if database.connected && redis.connected {
        "healthy"
    } else if database.connected || redis.connected {
        "degraded"
    } else {
        "unhealthy"
    };

    let health = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database,
        redis,
    };

    Json(JsonResponse::success(health))
}
