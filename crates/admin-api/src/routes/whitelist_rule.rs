use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use karateway_core::{
    models::{CreateWhitelistRuleRequest, UpdateWhitelistRuleRequest, WhitelistRule},
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
        .route("/", post(create_rule))
        .route("/", get(list_rules))
        .route("/{id}", get(get_rule))
        .route("/{id}", put(update_rule))
        .route("/{id}", delete(delete_rule))
}

#[utoipa::path(
    post,
    path = "/api/whitelist",
    request_body = CreateWhitelistRuleRequest,
    responses(
        (status = 201, description = "Whitelist rule created successfully", body = JsonResponse<WhitelistRule>),
        (status = 400, description = "Invalid request")
    ),
    tag = "whitelist-rules"
)]
async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateWhitelistRuleRequest>,
) -> ApiResult<(StatusCode, Json<JsonResponse<WhitelistRule>>)> {
    // Validate request
    req.validate()?;

    // Create rule
    let rule = state.whitelist_rule_repo.create(req).await?;

    Ok((
        StatusCode::CREATED,
        Json(JsonResponse::created(
            rule,
            "Whitelist rule created successfully",
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/whitelist",
    params(ListQuery),
    responses(
        (status = 200, description = "List of whitelist rules", body = JsonResponse<Vec<WhitelistRule>>)
    ),
    tag = "whitelist-rules"
)]
async fn list_rules(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<JsonResponse<Vec<WhitelistRule>>>> {
    let rules = state
        .whitelist_rule_repo
        .list(query.page, query.limit)
        .await?;

    let total = state.whitelist_rule_repo.count().await?;

    let meta = MetaResponse::new(query.page, query.limit, total);

    Ok(Json(JsonResponse::success_paginated(rules, meta)))
}

#[utoipa::path(
    get,
    path = "/api/whitelist/{id}",
    params(
        ("id" = Uuid, Path, description = "Whitelist rule ID")
    ),
    responses(
        (status = 200, description = "Whitelist rule found", body = JsonResponse<WhitelistRule>),
        (status = 404, description = "Whitelist rule not found")
    ),
    tag = "whitelist-rules"
)]
async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<JsonResponse<WhitelistRule>>> {
    let rule = state.whitelist_rule_repo.find_by_id(id).await?;

    Ok(Json(JsonResponse::success(rule)))
}

#[utoipa::path(
    put,
    path = "/api/whitelist/{id}",
    params(
        ("id" = Uuid, Path, description = "Whitelist rule ID")
    ),
    request_body = UpdateWhitelistRuleRequest,
    responses(
        (status = 200, description = "Whitelist rule updated", body = JsonResponse<WhitelistRule>),
        (status = 404, description = "Whitelist rule not found")
    ),
    tag = "whitelist-rules"
)]
async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWhitelistRuleRequest>,
) -> ApiResult<Json<JsonResponse<WhitelistRule>>> {
    // Validate request
    req.validate()?;

    // Update rule
    let rule = state.whitelist_rule_repo.update(id, req).await?;

    Ok(Json(JsonResponse::success_with_message(
        rule,
        "Whitelist rule updated successfully",
    )))
}

#[utoipa::path(
    delete,
    path = "/api/whitelist/{id}",
    params(
        ("id" = Uuid, Path, description = "Whitelist rule ID")
    ),
    responses(
        (status = 200, description = "Whitelist rule deleted"),
        (status = 404, description = "Whitelist rule not found")
    ),
    tag = "whitelist-rules"
)]
async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<JsonResponse<()>>)> {
    state.whitelist_rule_repo.delete(id).await?;

    Ok((StatusCode::OK, Json(JsonResponse::no_content())))
}
