use actix_web::{HttpResponse, delete, get, post, put, web};

use crate::app::AppState;
use crate::http::error::{created, deleted, service_error, updated};
use crate::http::types::{IdResponse, PagingQuery, TargetPage, TargetPayload};

#[utoipa::path(get, operation_id = "listTargets", path = "/api/targets", params(PagingQuery), responses((status = 200, body = TargetPage)))]
#[get("/api/targets")]
pub async fn list(query: web::Query<PagingQuery>, state: web::Data<AppState>) -> HttpResponse {
    match state.services.list_targets(query.pagination()).await {
        Ok(page) => HttpResponse::Ok().json(page),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(post, operation_id = "createTarget", path = "/api/targets", request_body = TargetPayload, responses((status = 201, body = IdResponse)))]
#[post("/api/targets")]
pub async fn create(payload: web::Json<TargetPayload>, state: web::Data<AppState>) -> HttpResponse {
    let payload = payload.into_inner();
    created(
        state
            .services
            .create_target(
                &payload.name,
                &payload.base_url,
                payload.enabled.unwrap_or(true),
                payload.timeout_ms.unwrap_or(10_000),
                payload.note.as_deref(),
            )
            .await,
    )
}

#[utoipa::path(put, operation_id = "updateTarget", path = "/api/targets/{id}", params(("id" = i64, Path)), request_body = TargetPayload, responses((status = 200, body = IdResponse), (status = 404)))]
#[put("/api/targets/{id}")]
pub async fn update(
    id: web::Path<i64>,
    payload: web::Json<TargetPayload>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let payload = payload.into_inner();
    updated(
        state
            .services
            .update_target(
                id.into_inner(),
                &payload.name,
                &payload.base_url,
                payload.enabled.unwrap_or(true),
                payload.timeout_ms.unwrap_or(10_000),
                payload.note.as_deref(),
            )
            .await,
    )
}

#[utoipa::path(delete, operation_id = "deleteTarget", path = "/api/targets/{id}", params(("id" = i64, Path)), responses((status = 204), (status = 404), (status = 409)))]
#[delete("/api/targets/{id}")]
pub async fn delete(id: web::Path<i64>, state: web::Data<AppState>) -> HttpResponse {
    deleted(state.services.delete_target(id.into_inner()).await)
}
