use actix_web::{HttpResponse, delete, get, web};

use crate::app::AppState;
use crate::http::error::{deleted, service_error};
use crate::http::types::{LogPage, LogQuery, PurgeResponse};

#[utoipa::path(get, operation_id = "listLogs", path = "/api/logs", params(LogQuery), responses((status = 200, body = LogPage)))]
#[get("/api/logs")]
pub async fn list(query: web::Query<LogQuery>, state: web::Data<AppState>) -> HttpResponse {
    match state
        .services
        .list_logs(query.into_inner().into_service_query())
        .await
    {
        Ok(page) => HttpResponse::Ok().json(page),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(delete, operation_id = "purgeLogs", path = "/api/logs", params(LogQuery), responses((status = 200, body = PurgeResponse)))]
#[delete("/api/logs")]
pub async fn purge(query: web::Query<LogQuery>, state: web::Data<AppState>) -> HttpResponse {
    match state
        .services
        .purge_logs(query.into_inner().into_service_query())
        .await
    {
        Ok(deleted) => HttpResponse::Ok().json(PurgeResponse { deleted }),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(get, operation_id = "getLog", path = "/api/logs/{id}", params(("id" = i64, Path)), responses((status = 200, body = crate::domain::LogDetail), (status = 404)))]
#[get("/api/logs/{id}")]
pub async fn detail(id: web::Path<i64>, state: web::Data<AppState>) -> HttpResponse {
    match state.services.log_detail(id.into_inner()).await {
        Ok(Some(value)) => HttpResponse::Ok().json(value),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(delete, operation_id = "deleteLog", path = "/api/logs/{id}", params(("id" = i64, Path)), responses((status = 204), (status = 404)))]
#[delete("/api/logs/{id}")]
pub async fn delete(id: web::Path<i64>, state: web::Data<AppState>) -> HttpResponse {
    deleted(state.services.delete_log(id.into_inner()).await)
}
