use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use karateway_core::{
    models::{ApiRoute, BackendService, CreateBackendServiceRequest, UpdateBackendServiceRequest},
    JsonResponse, MetaResponse,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::{error::ApiResult, routes::service_health, state::AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct BackendServiceWithRoutes {
    #[serde(flatten)]
    pub service: BackendService,
    pub routes: Vec<ApiRoute>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_service))
        .route("/", get(list_services))
        .route("/{id}", get(get_service))
        .route("/{id}", put(update_service))
        .route("/{id}", delete(delete_service))
        .route("/{id}/routes", get(get_service_with_routes))
}

#[utoipa::path(
    post,
    path = "/api/services",
    request_body = CreateBackendServiceRequest,
    responses(
        (status = 201, description = "Backend service created successfully", body = JsonResponse<BackendService>),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Service with same name already exists")
    ),
    tag = "backend-services"
)]
async fn create_service(
    State(state): State<AppState>,
    Json(req): Json<CreateBackendServiceRequest>,
) -> ApiResult<(StatusCode, Json<JsonResponse<BackendService>>)> {
    // Validate request
    req.validate()?;

    // Check if service with same name exists
    if let Some(_existing) = state.backend_service_repo.find_by_name(&req.name).await? {
        return Err(karateway_core::KaratewayError::Conflict(format!(
            "Backend service with name '{}' already exists",
            req.name
        ))
        .into());
    }

    // Create service
    let service = state.backend_service_repo.create(req).await?;

    // Trigger health check for the new service (async, don't wait)
    let state_clone = state.clone();
    let service_id = service.id.to_string();
    tokio::spawn(async move {
        if let Some(health) = service_health::check_service_health(&state_clone, &service_id).await
        {
            tracing::info!(
                "Initial health check for service {}: {}",
                health.name,
                health.status_message
            );

            // Invalidate cache to force refresh on next request
            if let Ok(mut redis_conn) = state_clone.redis_pool.get().await {
                let _: Result<(), _> = redis_conn.del("services:health:data").await;
                tracing::debug!("Invalidated health cache after creating new service");
            }
        }
    });

    Ok((
        StatusCode::CREATED,
        Json(JsonResponse::created(
            service,
            "Backend service created successfully",
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/services",
    params(ListQuery),
    responses(
        (status = 200, description = "List of backend services", body = JsonResponse<Vec<BackendService>>)
    ),
    tag = "backend-services"
)]
async fn list_services(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<JsonResponse<Vec<BackendService>>>> {
    let services = state
        .backend_service_repo
        .list(query.page, query.limit)
        .await?;

    let total = state.backend_service_repo.count().await?;

    let meta = MetaResponse::new(query.page, query.limit, total);

    Ok(Json(JsonResponse::success_paginated(services, meta)))
}

#[utoipa::path(
    get,
    path = "/api/services/{id}",
    params(
        ("id" = Uuid, Path, description = "Backend service ID")
    ),
    responses(
        (status = 200, description = "Backend service found", body = JsonResponse<BackendService>),
        (status = 404, description = "Backend service not found")
    ),
    tag = "backend-services"
)]
async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<JsonResponse<BackendService>>> {
    let service = state.backend_service_repo.find_by_id(id).await?;

    Ok(Json(JsonResponse::success(service)))
}

#[utoipa::path(
    put,
    path = "/api/services/{id}",
    params(
        ("id" = Uuid, Path, description = "Backend service ID")
    ),
    request_body = UpdateBackendServiceRequest,
    responses(
        (status = 200, description = "Backend service updated", body = JsonResponse<BackendService>),
        (status = 404, description = "Backend service not found")
    ),
    tag = "backend-services"
)]
async fn update_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBackendServiceRequest>,
) -> ApiResult<Json<JsonResponse<BackendService>>> {
    // Validate request
    req.validate()?;

    // Update service
    let service = state.backend_service_repo.update(id, req).await?;

    Ok(Json(JsonResponse::success_with_message(
        service,
        "Backend service updated successfully",
    )))
}

#[utoipa::path(
    delete,
    path = "/api/services/{id}",
    params(
        ("id" = Uuid, Path, description = "Backend service ID")
    ),
    responses(
        (status = 200, description = "Backend service deleted"),
        (status = 404, description = "Backend service not found")
    ),
    tag = "backend-services"
)]
async fn delete_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<JsonResponse<()>>)> {
    state.backend_service_repo.delete(id).await?;

    Ok((StatusCode::OK, Json(JsonResponse::no_content())))
}

#[utoipa::path(
    get,
    path = "/api/services/{id}/routes",
    params(
        ("id" = Uuid, Path, description = "Backend service ID")
    ),
    responses(
        (status = 200, description = "Backend service with routes", body = JsonResponse<BackendServiceWithRoutes>),
        (status = 404, description = "Backend service not found")
    ),
    tag = "backend-services"
)]
async fn get_service_with_routes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<JsonResponse<BackendServiceWithRoutes>>> {
    // Get the service
    let service = state.backend_service_repo.find_by_id(id).await?;

    // Get all routes for this service using the repository
    let routes = state.api_route_repo.list_by_backend_service(id).await?;

    let response = BackendServiceWithRoutes { service, routes };

    Ok(Json(JsonResponse::success(response)))
}
