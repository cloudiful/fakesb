use actix_web::{HttpResponse, get, post, web};

use crate::app::AppState;
use crate::http::error::service_error;

#[utoipa::path(get, operation_id = "exportConfig", path = "/api/export", responses((status = 200, body = crate::domain::ExportBundle)))]
#[get("/api/export")]
pub async fn export_config(state: web::Data<AppState>) -> HttpResponse {
    match state.services.export_config().await {
        Ok(bundle) => HttpResponse::Ok().json(bundle),
        Err(error) => service_error(error),
    }
}

#[utoipa::path(post, operation_id = "importConfig", path = "/api/import", request_body = crate::domain::ExportBundle, responses((status = 200, body = crate::domain::ImportSummary)))]
#[post("/api/import")]
pub async fn import_config(
    payload: web::Json<crate::domain::ExportBundle>,
    state: web::Data<AppState>,
) -> HttpResponse {
    match state.services.import_config(&payload.into_inner()).await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(error) => service_error(error),
    }
}
