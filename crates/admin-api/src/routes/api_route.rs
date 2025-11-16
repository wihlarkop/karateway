use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use karateway_core::{
    models::{ApiRoute, CreateApiRouteRequest, UpdateApiRouteRequest},
    JsonResponse, MetaResponse,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;
use validator::Validate;

use crate::{error::ApiResult, state::AppState};

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
        .route("/", post(create_route))
        .route("/", get(list_routes))
        .route("/{id}", get(get_route))
        .route("/{id}", put(update_route))
        .route("/{id}", delete(delete_route))
}

#[utoipa::path(
    post,
    path = "/api/routes",
    request_body = CreateApiRouteRequest,
    responses(
        (status = 201, description = "API route created successfully", body = JsonResponse<ApiRoute>),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Backend service not found")
    ),
    tag = "api-routes"
)]
async fn create_route(
    State(state): State<AppState>,
    Json(req): Json<CreateApiRouteRequest>,
) -> ApiResult<(StatusCode, Json<JsonResponse<ApiRoute>>)> {
    // Validate request
    req.validate()?;

    // Verify backend service exists
    state
        .backend_service_repo
        .find_by_id(req.backend_service_id)
        .await?;

    // Create route
    let route = state.api_route_repo.create(req).await?;

    Ok((
        StatusCode::CREATED,
        Json(JsonResponse::created(
            route,
            "API route created successfully",
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/routes",
    params(ListQuery),
    responses(
        (status = 200, description = "List of API routes", body = JsonResponse<Vec<ApiRoute>>)
    ),
    tag = "api-routes"
)]
async fn list_routes(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<JsonResponse<Vec<ApiRoute>>>> {
    let routes = state.api_route_repo.list(query.page, query.limit).await?;

    let total = state.api_route_repo.count().await?;

    let meta = MetaResponse::new(query.page, query.limit, total);

    Ok(Json(JsonResponse::success_paginated(routes, meta)))
}

#[utoipa::path(
    get,
    path = "/api/routes/{id}",
    params(
        ("id" = Uuid, Path, description = "API route ID")
    ),
    responses(
        (status = 200, description = "API route found", body = JsonResponse<ApiRoute>),
        (status = 404, description = "API route not found")
    ),
    tag = "api-routes"
)]
async fn get_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<JsonResponse<ApiRoute>>> {
    let route = state.api_route_repo.find_by_id(id).await?;

    Ok(Json(JsonResponse::success(route)))
}

#[utoipa::path(
    put,
    path = "/api/routes/{id}",
    params(
        ("id" = Uuid, Path, description = "API route ID")
    ),
    request_body = UpdateApiRouteRequest,
    responses(
        (status = 200, description = "API route updated", body = JsonResponse<ApiRoute>),
        (status = 404, description = "API route not found")
    ),
    tag = "api-routes"
)]
async fn update_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateApiRouteRequest>,
) -> ApiResult<Json<JsonResponse<ApiRoute>>> {
    // Validate request
    req.validate()?;

    // If backend_service_id is being updated, verify it exists
    if let Some(backend_service_id) = req.backend_service_id {
        state
            .backend_service_repo
            .find_by_id(backend_service_id)
            .await?;
    }

    // Update route
    let route = state.api_route_repo.update(id, req).await?;

    Ok(Json(JsonResponse::success_with_message(
        route,
        "API route updated successfully",
    )))
}

#[utoipa::path(
    delete,
    path = "/api/routes/{id}",
    params(
        ("id" = Uuid, Path, description = "API route ID")
    ),
    responses(
        (status = 200, description = "API route deleted"),
        (status = 404, description = "API route not found")
    ),
    tag = "api-routes"
)]
async fn delete_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<JsonResponse<()>>)> {
    state.api_route_repo.delete(id).await?;

    Ok((StatusCode::OK, Json(JsonResponse::no_content())))
}
