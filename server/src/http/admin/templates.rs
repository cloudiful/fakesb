use actix_web::{HttpResponse, get, post, put, web};

use crate::app::AppState;
use crate::http::error::{created, service_error, updated};
use crate::http::types::{IdResponse, PagingQuery, TemplatePage, TemplatePayload};

#[utoipa::path(get, operation_id = "listTemplates", path = "/api/templates", params(PagingQuery), responses((status = 200, body = TemplatePage)))]
#[get("/api/templates")]
pub async fn list(query: web::Query<PagingQuery>, state: web::Data<AppState>) -> HttpResponse {
    match state.services.list_templates(query.pagination()).await {
        Ok(page) => HttpResponse::Ok().json(page),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(post, operation_id = "createTemplate", path = "/api/templates", request_body = TemplatePayload, responses((status = 201, body = IdResponse)))]
#[post("/api/templates")]
pub async fn create(
    payload: web::Json<TemplatePayload>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let payload = payload.into_inner();
    let headers = payload
        .headers
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));
    created(
        state
            .services
            .create_template(
                &payload.name,
                payload.content_type.as_deref().unwrap_or("text/plain"),
                &payload.raw_template,
                payload.format.as_deref().unwrap_or("text"),
                payload.status_code.unwrap_or(200),
                &headers,
                payload.enabled.unwrap_or(true),
                payload.note.as_deref(),
            )
            .await,
    )
}

#[utoipa::path(put, operation_id = "updateTemplate", path = "/api/templates/{id}", params(("id" = i64, Path)), request_body = TemplatePayload, responses((status = 200, body = IdResponse), (status = 404)))]
#[put("/api/templates/{id}")]
pub async fn update(
    id: web::Path<i64>,
    payload: web::Json<TemplatePayload>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let payload = payload.into_inner();
    let headers = payload
        .headers
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));
    updated(
        state
            .services
            .update_template(
                id.into_inner(),
                &payload.name,
                payload.content_type.as_deref().unwrap_or("text/plain"),
                &payload.raw_template,
                payload.format.as_deref().unwrap_or("text"),
                payload.status_code.unwrap_or(200),
                &headers,
                payload.enabled.unwrap_or(true),
                payload.note.as_deref(),
            )
            .await,
    )
}
