use crate::domain::{EsbResponse, ParsedEsbMessage};
use crate::repositories;
use crate::service::{AppServices, ServiceError};

pub(super) async fn render(
    services: &AppServices,
    request: &ParsedEsbMessage,
    template_id: Option<i64>,
) -> Result<EsbResponse, ServiceError> {
    let template_id =
        template_id.ok_or_else(|| ServiceError::Template("mock rule missing template".into()))?;
    let template = repositories::templates::find_enabled(services.pool(), template_id)
        .await?
        .ok_or_else(|| ServiceError::Template("response template is missing or disabled".into()))?;
    let raw_body = crate::template::render(&template.raw_template, request)?;
    let normalized_json = xml::convert::to_json(&raw_body)
        .ok()
        .map(|json| crate::http::json::simplify(&json));
    let (ret_code, ret_msg) = super::response::extract_ret_fields(normalized_json.as_ref());

    Ok(EsbResponse {
        status_code: 200,
        content_type: template.content_type,
        raw_body,
        normalized_json,
        ret_code,
        ret_msg,
    })
}
