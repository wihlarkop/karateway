use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use karateway_core::{
    models::{CreateRateLimitRequest, RateLimit, UpdateRateLimitRequest},
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
        .route("/", post(create_limit))
        .route("/", get(list_limits))
        .route("/{id}", get(get_limit))
        .route("/{id}", put(update_limit))
        .route("/{id}", delete(delete_limit))
}

#[utoipa::path(
    post,
    path = "/api/rate-limits",
    request_body = CreateRateLimitRequest,
    responses(
        (status = 201, description = "Rate limit created successfully", body = JsonResponse<RateLimit>),
        (status = 400, description = "Invalid request")
    ),
    tag = "rate-limits"
)]
async fn create_limit(
    State(state): State<AppState>,
    Json(req): Json<CreateRateLimitRequest>,
) -> ApiResult<(StatusCode, Json<JsonResponse<RateLimit>>)> {
    // Validate request
    req.validate()?;

    // Create limit
    let limit = state.rate_limit_repo.create(req).await?;

    Ok((
        StatusCode::CREATED,
        Json(JsonResponse::created(
            limit,
            "Rate limit created successfully",
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/rate-limits",
    params(ListQuery),
    responses(
        (status = 200, description = "List of rate limits", body = JsonResponse<Vec<RateLimit>>)
    ),
    tag = "rate-limits"
)]
async fn list_limits(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<JsonResponse<Vec<RateLimit>>>> {
    let limits = state.rate_limit_repo.list(query.page, query.limit).await?;

    let total = state.rate_limit_repo.count().await?;

    let meta = MetaResponse::new(query.page, query.limit, total);

    Ok(Json(JsonResponse::success_paginated(limits, meta)))
}

#[utoipa::path(
    get,
    path = "/api/rate-limits/{id}",
    params(
        ("id" = Uuid, Path, description = "Rate limit ID")
    ),
    responses(
        (status = 200, description = "Rate limit found", body = JsonResponse<RateLimit>),
        (status = 404, description = "Rate limit not found")
    ),
    tag = "rate-limits"
)]
async fn get_limit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<JsonResponse<RateLimit>>> {
    let limit = state.rate_limit_repo.find_by_id(id).await?;

    Ok(Json(JsonResponse::success(limit)))
}

#[utoipa::path(
    put,
    path = "/api/rate-limits/{id}",
    params(
        ("id" = Uuid, Path, description = "Rate limit ID")
    ),
    request_body = UpdateRateLimitRequest,
    responses(
        (status = 200, description = "Rate limit updated", body = JsonResponse<RateLimit>),
        (status = 404, description = "Rate limit not found")
    ),
    tag = "rate-limits"
)]
async fn update_limit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRateLimitRequest>,
) -> ApiResult<Json<JsonResponse<RateLimit>>> {
    // Validate request
    req.validate()?;

    // Update limit
    let limit = state.rate_limit_repo.update(id, req).await?;

    Ok(Json(JsonResponse::success_with_message(
        limit,
        "Rate limit updated successfully",
    )))
}

#[utoipa::path(
    delete,
    path = "/api/rate-limits/{id}",
    params(
        ("id" = Uuid, Path, description = "Rate limit ID")
    ),
    responses(
        (status = 200, description = "Rate limit deleted"),
        (status = 404, description = "Rate limit not found")
    ),
    tag = "rate-limits"
)]
async fn delete_limit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<JsonResponse<()>>)> {
    state.rate_limit_repo.delete(id).await?;

    Ok((StatusCode::OK, Json(JsonResponse::no_content())))
}
