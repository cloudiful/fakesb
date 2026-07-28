use crate::domain::{EsbResponse, ParsedEsbMessage};
use crate::repositories;
use crate::service::{AppServices, ServiceError};

pub(super) async fn execute(
    services: &AppServices,
    request: &ParsedEsbMessage,
    target_id: Option<i64>,
) -> Result<EsbResponse, ServiceError> {
    let target_id = target_id.ok_or_else(|| ServiceError::Remote("no target configured".into()))?;
    let target = repositories::targets::find_enabled(services.pool(), target_id)
        .await?
        .ok_or_else(|| ServiceError::Remote("target is missing or disabled".into()))?;
    let timeout_ms = if target.timeout_ms > 0 {
        target.timeout_ms as u64
    } else {
        services.request_timeout_ms()
    };
    let response = services
        .client()
        .post(&target.base_url)
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .header("Content-Type", "application/xml")
        .body(request.raw_body.clone())
        .send()
        .await
        .map_err(|err| ServiceError::Remote(err.to_string()))?;

    let status_code = response.status().as_u16();
    let raw_body = response
        .text()
        .await
        .map_err(|err| ServiceError::Remote(err.to_string()))?;
    if status_code >= 400 {
        return Err(ServiceError::Remote(format!(
            "remote returned HTTP {status_code}: {raw_body}"
        )));
    }
    let normalized_json = xml::convert::to_json(&raw_body)
        .ok()
        .map(|json| crate::http::json::simplify(&json));
    let (ret_code, ret_msg) = super::response::extract_ret_fields(normalized_json.as_ref());

    Ok(EsbResponse {
        status_code,
        content_type: "application/xml".into(),
        raw_body,
        normalized_json,
        ret_code,
        ret_msg,
    })
}
