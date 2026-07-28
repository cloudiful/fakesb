use crate::domain::{MockRequest, ResponseTemplate};
use crate::repositories;
use crate::service::{AppServices, ServiceError};

pub(super) async fn render(
    services: &AppServices,
    request: &MockRequest,
    template_id: Option<i64>,
) -> Result<crate::domain::MockResponse, ServiceError> {
    let template_id = template_id
        .ok_or_else(|| ServiceError::Template("static rule missing response template".into()))?;
    let template = repositories::templates::find_enabled(services.pool(), template_id)
        .await?
        .ok_or_else(|| ServiceError::Template("response template is missing or disabled".into()))?;
    render_template(&template, request)
}

fn render_template(
    template: &ResponseTemplate,
    request: &MockRequest,
) -> Result<crate::domain::MockResponse, ServiceError> {
    let raw_body =
        crate::template::render_for_format(&template.raw_template, &template.format, request)?;
    let headers =
        crate::domain::json_to_headers(&template.headers).map_err(ServiceError::Validation)?;
    Ok(crate::service::response::from_body(
        template.status_code,
        template.content_type.clone(),
        headers,
        raw_body,
    ))
}
