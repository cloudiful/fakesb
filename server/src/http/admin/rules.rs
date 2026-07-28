use actix_web::{HttpResponse, get, post, put, web};

use crate::app::AppState;
use crate::http::error::{created, service_error, updated};
use crate::http::types::{IdResponse, PagingQuery, RulePage, RulePayload};
use crate::service::RuleInput;

#[utoipa::path(get, operation_id = "listRules", path = "/api/rules", params(PagingQuery), responses((status = 200, body = RulePage)))]
#[get("/api/rules")]
pub async fn list(query: web::Query<PagingQuery>, state: web::Data<AppState>) -> HttpResponse {
    match state.services.list_rules(query.pagination()).await {
        Ok(page) => HttpResponse::Ok().json(page),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(post, operation_id = "createRule", path = "/api/rules", request_body = RulePayload, responses((status = 201, body = IdResponse)))]
#[post("/api/rules")]
pub async fn create(payload: web::Json<RulePayload>, state: web::Data<AppState>) -> HttpResponse {
    let payload = payload.into_inner();
    created(state.services.create_rule(input(&payload)).await)
}

#[utoipa::path(put, operation_id = "updateRule", path = "/api/rules/{id}", params(("id" = i64, Path)), request_body = RulePayload, responses((status = 200, body = IdResponse), (status = 404)))]
#[put("/api/rules/{id}")]
pub async fn update(
    id: web::Path<i64>,
    payload: web::Json<RulePayload>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let payload = payload.into_inner();
    updated(
        state
            .services
            .update_rule(id.into_inner(), input(&payload))
            .await,
    )
}

fn input(payload: &RulePayload) -> RuleInput<'_> {
    RuleInput {
        service_code: &payload.service_code,
        message_type: &payload.message_type,
        message_code: &payload.message_code,
        target_id: payload.target_id,
        mode: payload.mode,
        response_template_id: payload.response_template_id,
        priority: payload.priority.unwrap_or(0),
        enabled: payload.enabled.unwrap_or(true),
        note: payload.note.as_deref(),
    }
}
