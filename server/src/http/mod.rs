pub mod admin;
pub mod json;

mod error;
pub(crate) mod types;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder, get, post, web};
use utoipa::OpenApi;

use crate::app::AppState;
use crate::service::{self, ServiceError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz)
        .service(dispatch_esb)
        .service(openapi_json)
        .service(admin::targets::list)
        .service(admin::targets::create)
        .service(admin::targets::update)
        .service(admin::rules::list)
        .service(admin::rules::create)
        .service(admin::rules::update)
        .service(admin::templates::list)
        .service(admin::templates::create)
        .service(admin::templates::update)
        .service(admin::logs::list)
        .service(admin::logs::detail);
}

#[utoipa::path(
    get,
    operation_id = "healthz",
    path = "/healthz",
    responses((status = 200, description = "Service is healthy"))
)]
#[get("/healthz")]
pub async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[utoipa::path(
    post,
    operation_id = "dispatchEsb",
    path = "/Esbhttp/SmartEBANK",
    request_body(content = String, content_type = "application/xml"),
    responses((status = 200, description = "ESB XML response", content_type = "application/xml"))
)]
#[post("/Esbhttp/SmartEBANK")]
pub async fn dispatch_esb(body: web::Bytes, state: web::Data<AppState>) -> impl Responder {
    let body = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(error) => return xml_error(ServiceError::Parse(error.to_string())),
    };
    match service::dispatch(&state.services, &body).await {
        Ok(response) => HttpResponse::build(
            StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK),
        )
        .content_type(response.content_type)
        .body(response.raw_body),
        Err(error) => xml_error(error),
    }
}

#[utoipa::path(
    get,
    operation_id = "getOpenApi",
    path = "/api/openapi.json",
    responses((status = 200))
)]
#[get("/api/openapi.json")]
pub async fn openapi_json() -> impl Responder {
    HttpResponse::Ok().json(crate::openapi::ApiDoc::openapi())
}

fn xml_error(error: ServiceError) -> HttpResponse {
    let status = match &error {
        ServiceError::Parse(_) | ServiceError::Validation(_) => StatusCode::BAD_REQUEST,
        ServiceError::Remote(_) => StatusCode::BAD_GATEWAY,
        ServiceError::Template(_) | ServiceError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = error.to_string();
    HttpResponse::build(status)
        .content_type("application/xml")
        .body(format!(
            "<error><ret_code>{}</ret_code><ret_msg>{}</ret_msg></error>",
            status.as_u16(),
            xml_escape(&message)
        ))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
