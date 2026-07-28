use actix_web::HttpResponse;

use crate::service::ServiceError;

pub fn service_error(error: ServiceError) -> HttpResponse {
    let status = match &error {
        ServiceError::Validation(_) | ServiceError::Parse(_) => {
            actix_web::http::StatusCode::BAD_REQUEST
        }
        ServiceError::Remote(_) => actix_web::http::StatusCode::BAD_GATEWAY,
        ServiceError::Template(_) | ServiceError::Database(_) => {
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
        ServiceError::NoMatch => actix_web::http::StatusCode::NOT_FOUND,
    };
    HttpResponse::build(status).json(serde_json::json!({"error": error.to_string()}))
}

pub fn created(id: Result<i64, ServiceError>) -> HttpResponse {
    match id {
        Ok(id) => HttpResponse::Created().json(super::types::IdResponse { id }),
        Err(error) => service_error(error),
    }
}

pub fn updated(id: Result<Option<i64>, ServiceError>) -> HttpResponse {
    match id {
        Ok(Some(id)) => HttpResponse::Ok().json(super::types::IdResponse { id }),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(error) => service_error(error),
    }
}
