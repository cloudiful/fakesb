pub mod admin;

mod error;
pub(crate) mod types;

use actix_web::http::StatusCode;
use actix_web::http::header::{ACCEPT, CONTENT_TYPE, HeaderName, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use utoipa::OpenApi;

use crate::app::AppState;
use crate::service::{self, ServiceError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz)
        .service(openapi_json)
        .service(admin::targets::list)
        .service(admin::targets::create)
        .service(admin::targets::update)
        .service(admin::targets::delete)
        .service(admin::rules::list)
        .service(admin::rules::create)
        .service(admin::rules::test)
        .service(admin::rules::update)
        .service(admin::rules::delete)
        .service(admin::templates::list)
        .service(admin::templates::create)
        .service(admin::templates::update)
        .service(admin::templates::delete)
        .service(admin::logs::list)
        .service(admin::logs::detail)
        .service(admin::logs::purge)
        .service(admin::logs::delete)
        .service(admin::config::export_config)
        .service(admin::config::import_config)
        .default_service(web::route().to(dispatch));
}

#[utoipa::path(
    get,
    operation_id = "healthz",
    path = "/healthz",
    responses((status = 200, description = "Service is healthy"))
)]
#[actix_web::get("/healthz")]
pub async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[utoipa::path(
    get,
    operation_id = "getOpenApi",
    path = "/api/openapi.json",
    responses((status = 200))
)]
#[actix_web::get("/api/openapi.json")]
pub async fn openapi_json() -> impl Responder {
    HttpResponse::Ok().json(crate::openapi::ApiDoc::openapi())
}

pub async fn dispatch(
    request: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    let parsed = match service::parse_request(
        request.method().as_str(),
        request.uri().to_string().as_str(),
        request.headers(),
        &body,
    ) {
        Ok(parsed) => parsed,
        Err(error) => return request_error(error, &request),
    };

    match service::dispatch(&state.services, &parsed).await {
        Ok(response) => {
            let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK);
            let mut builder = HttpResponse::build(status);
            for (name, value) in response.headers {
                if name.eq_ignore_ascii_case("content-type") {
                    continue;
                }
                if let (Ok(name), Ok(value)) =
                    (HeaderName::try_from(name), HeaderValue::try_from(value))
                {
                    builder.insert_header((name, value));
                }
            }
            builder
                .content_type(response.content_type)
                .body(response.raw_body)
        }
        Err(error) => request_error(error, &request),
    }
}

fn request_error(error: ServiceError, request: &HttpRequest) -> HttpResponse {
    let status = match &error {
        ServiceError::Parse(_) | ServiceError::Validation(_) => StatusCode::BAD_REQUEST,
        ServiceError::Conflict(_) => StatusCode::CONFLICT,
        ServiceError::NoMatch => StatusCode::NOT_FOUND,
        ServiceError::Remote(_) => StatusCode::BAD_GATEWAY,
        ServiceError::Template(_) | ServiceError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = error.to_string();
    if prefers_json(request) {
        return HttpResponse::build(status)
            .content_type("application/json")
            .body(serde_json::json!({"error": message}).to_string());
    }
    if prefers_xml(request) {
        return HttpResponse::build(status)
            .content_type("application/xml")
            .body(format!(
                "<error><status>{}</status><message>{}</message></error>",
                status.as_u16(),
                xml_escape(&message)
            ));
    }
    HttpResponse::build(status)
        .content_type("text/plain")
        .body(message)
}

fn prefers_json(request: &HttpRequest) -> bool {
    request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .ends_with("json")
        })
        || request
            .headers()
            .get(ACCEPT)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("json"))
}

fn prefers_xml(request: &HttpRequest) -> bool {
    request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("xml"))
        || request
            .headers()
            .get(ACCEPT)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("xml"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
