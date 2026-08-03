use actix_web::{HttpResponse, delete, get, post, put, web};

use crate::app::AppState;
use crate::http::error::{created, deleted, service_error, updated};
use crate::http::types::{IdResponse, PagingQuery, RulePage, RulePayload, RuleTestPayload};
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

#[utoipa::path(post, operation_id = "testRule", path = "/api/rules/test", request_body = RuleTestPayload, responses((status = 200, body = crate::domain::RuleTestResult)))]
#[post("/api/rules/test")]
pub async fn test(payload: web::Json<RuleTestPayload>, state: web::Data<AppState>) -> HttpResponse {
    let payload = payload.into_inner();
    match state
        .services
        .test_rule(
            &payload.method,
            &payload.path,
            &payload.query,
            &payload.headers,
            payload.content_type.as_deref(),
            payload.body.as_deref().unwrap_or(""),
        )
        .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(error) => service_error(error),
    }
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

#[utoipa::path(delete, operation_id = "deleteRule", path = "/api/rules/{id}", params(("id" = i64, Path)), responses((status = 204), (status = 404)))]
#[delete("/api/rules/{id}")]
pub async fn delete(id: web::Path<i64>, state: web::Data<AppState>) -> HttpResponse {
    deleted(state.services.delete_rule(id.into_inner()).await)
}

fn input(payload: &RulePayload) -> RuleInput<'_> {
    RuleInput {
        matcher: &payload.matcher,
        target_id: payload.target_id,
        action: payload.action,
        response_template_id: payload.response_template_id,
        delay_ms: payload.delay_ms.unwrap_or(0),
        sequence_mode: payload.sequence_mode.unwrap_or(false),
        sequence_steps: payload
            .sequence_steps
            .iter()
            .map(|step| crate::service::SequenceStepInput {
                template_id: step.template_id,
            })
            .collect(),
        priority: payload.priority.unwrap_or(0),
        enabled: payload.enabled.unwrap_or(true),
        note: payload.note.as_deref(),
    }
}
