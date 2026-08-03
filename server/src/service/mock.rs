use crate::domain::{MockRequest, MockResponse, ResponseTemplate};
use crate::repositories;
use crate::service::{AppServices, ServiceError};

pub(super) async fn render(
    services: &AppServices,
    request: &MockRequest,
    template_id: Option<i64>,
) -> Result<MockResponse, ServiceError> {
    let template = find_template(services, template_id).await?;
    render_template(&template, request, None)
}

pub(super) async fn render_from_response(
    services: &AppServices,
    request: &MockRequest,
    template_id: Option<i64>,
    upstream: &MockResponse,
) -> Result<MockResponse, ServiceError> {
    let template = find_template(services, template_id).await?;
    render_template(&template, request, Some(upstream))
}

async fn find_template(
    services: &AppServices,
    template_id: Option<i64>,
) -> Result<ResponseTemplate, ServiceError> {
    let template_id = template_id
        .ok_or_else(|| ServiceError::Template("static rule missing response template".into()))?;
    repositories::templates::find_enabled(services.pool(), template_id)
        .await?
        .ok_or_else(|| ServiceError::Template("response template is missing or disabled".into()))
}

fn render_template(
    template: &ResponseTemplate,
    request: &MockRequest,
    upstream: Option<&MockResponse>,
) -> Result<MockResponse, ServiceError> {
    let raw_body = match upstream {
        Some(response) => crate::template::render_for_response(
            &template.raw_template,
            &template.format,
            request,
            response,
        )?,
        None => {
            crate::template::render_for_format(&template.raw_template, &template.format, request)?
        }
    };
    let headers =
        crate::domain::json_to_headers(&template.headers).map_err(ServiceError::Validation)?;
    Ok(crate::service::response::from_body(
        template.status_code,
        template.content_type.clone(),
        headers,
        raw_body,
    ))
}
