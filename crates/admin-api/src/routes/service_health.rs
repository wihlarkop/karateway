use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use karateway_core::JsonResponse;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use utoipa::ToSchema;
use uuid;

use crate::state::AppState;

const HEALTH_CACHE_KEY: &str = "services:health:data";
const HEALTH_CACHE_TTL: i64 = 12 * 60 * 60; // 12 hours in seconds

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceHealth {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub health_check_url: Option<String>,
    pub is_healthy: bool,
    pub status_message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServicesHealthResponse {
    pub services: Vec<ServiceHealth>,
    pub last_checked: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct HealthQueryParams {
    #[serde(default)]
    pub force_refresh: bool,
}

#[utoipa::path(
    get,
    path = "/api/services/health",
    params(
        ("force_refresh" = Option<bool>, Query, description = "Force refresh health check, bypassing cache")
    ),
    responses(
        (status = 200, description = "Backend service health statuses with last checked time", body = JsonResponse<ServicesHealthResponse>)
    ),
    tag = "services"
)]
pub async fn get_services_health(
    State(state): State<AppState>,
    Query(params): Query<HealthQueryParams>,
) -> Json<JsonResponse<ServicesHealthResponse>> {
    // Try to get cached data if not forcing refresh
    if !params.force_refresh {
        if let Ok(mut redis_conn) = state.redis_pool.get().await {
            if let Ok(Some(cached_json)) = redis_conn
                .get::<&str, Option<String>>(HEALTH_CACHE_KEY)
                .await
            {
                if let Ok(cached_response) =
                    serde_json::from_str::<ServicesHealthResponse>(&cached_json)
                {
                    tracing::debug!("Returning cached health check data from Redis");
                    return Json(JsonResponse::success(cached_response));
                }
            }
        }
    }
    // Get all backend services
    let services = match state.backend_service_repo.list(1, 100).await {
        Ok(services) => services,
        Err(e) => {
            return Json(JsonResponse {
                data: None,
                message: Some(format!("Failed to fetch services: {}", e)),
                success: false,
                meta: None,
                status_code: 500,
                timestamp: Utc::now(),
                error_code: Some("FETCH_ERROR".to_string()),
            })
        }
    };

    // Create HTTP client for health checks
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create HTTP client");

    let mut health_statuses = Vec::new();

    for service in services {
        let (is_healthy, status_message) = if let Some(ref health_url) = service.health_check_url {
            // Build full health check URL
            let full_url =
                if health_url.starts_with("http://") || health_url.starts_with("https://") {
                    health_url.clone()
                } else {
                    format!("{}{}", service.base_url, health_url)
                };

            // Perform health check
            match client.get(&full_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        (true, format!("Healthy ({})", response.status()))
                    } else {
                        (false, format!("Unhealthy - returned {}", response.status()))
                    }
                }
                Err(e) => (false, format!("Unhealthy - {}", e)),
            }
        } else {
            // No health check configured
            (true, "No health check configured".to_string())
        };

        health_statuses.push(ServiceHealth {
            id: service.id.to_string(),
            name: service.name,
            base_url: service.base_url,
            health_check_url: service.health_check_url,
            is_healthy,
            status_message,
        });
    }

    let response = ServicesHealthResponse {
        services: health_statuses,
        last_checked: Utc::now(),
    };

    // Cache the result in Redis with 12-hour TTL
    if let Ok(mut redis_conn) = state.redis_pool.get().await {
        if let Ok(json) = serde_json::to_string(&response) {
            let _: Result<(), _> = redis_conn
                .set_ex(HEALTH_CACHE_KEY, json, HEALTH_CACHE_TTL as u64)
                .await;
            tracing::debug!(
                "Cached health check data in Redis for {} seconds",
                HEALTH_CACHE_TTL
            );
        }
    }

    Json(JsonResponse::success(response))
}

/// Force health check for a specific service (used after creating new service)
pub async fn check_service_health(state: &AppState, service_id: &str) -> Option<ServiceHealth> {
    // Parse service_id to Uuid
    let id = uuid::Uuid::parse_str(service_id).ok()?;

    // Get the specific service
    let service = state.backend_service_repo.find_by_id(id).await.ok()?;

    let (is_healthy, status_message) = if let Some(health_url) = &service.health_check_url {
        let full_url = if health_url.starts_with("http://") || health_url.starts_with("https://") {
            health_url.clone()
        } else {
            format!("{}{}", service.base_url, health_url)
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .ok()?;

        match client.get(&full_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    (true, format!("Healthy ({})", response.status()))
                } else {
                    (false, format!("Unhealthy - returned {}", response.status()))
                }
            }
            Err(e) => (false, format!("Unhealthy - {}", e)),
        }
    } else {
        (true, "No health check configured".to_string())
    };

    Some(ServiceHealth {
        id: service.id.to_string(),
        name: service.name,
        base_url: service.base_url,
        health_check_url: service.health_check_url,
        is_healthy,
        status_message,
    })
}
