use std::time::Duration;

use crate::domain::{HeaderMap, MockRequest};
use crate::repositories;
use crate::service::{AppServices, ServiceError};

pub(super) async fn execute(
    services: &AppServices,
    request: &MockRequest,
    target_id: Option<i64>,
) -> Result<crate::domain::MockResponse, ServiceError> {
    let target_id = target_id.ok_or_else(|| ServiceError::Remote("no target configured".into()))?;
    let target = repositories::targets::find_enabled(services.pool(), target_id)
        .await?
        .ok_or_else(|| ServiceError::Remote("target is missing or disabled".into()))?;
    let timeout_ms = if target.timeout_ms > 0 {
        target.timeout_ms as u64
    } else {
        services.request_timeout_ms()
    };
    let url = target_url(&target.base_url, request)?;
    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|error| ServiceError::Remote(format!("invalid request method: {error}")))?;
    let mut builder = services
        .client()
        .request(method, url)
        .timeout(Duration::from_millis(timeout_ms));
    for (name, value) in &request.headers {
        if !is_hop_by_hop(name) && name != "host" && name != "content-length" {
            builder = builder.header(name, value);
        }
    }
    let response = builder
        .body(request.raw_body.clone())
        .send()
        .await
        .map_err(|error| ServiceError::Remote(error.to_string()))?;
    let status_code = response.status().as_u16();
    let headers = response_headers(response.headers());
    let content_type = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| "text/plain".into());
    let raw_body = response
        .text()
        .await
        .map_err(|error| ServiceError::Remote(error.to_string()))?;
    Ok(crate::service::response::from_body(
        status_code,
        content_type,
        headers,
        raw_body,
    ))
}

fn target_url(base_url: &str, request: &MockRequest) -> Result<reqwest::Url, ServiceError> {
    let mut url = reqwest::Url::parse(base_url)
        .map_err(|error| ServiceError::Remote(format!("invalid target URL: {error}")))?;
    let base_path = url.path().trim_end_matches('/');
    let request_path = request.path.trim_start_matches('/');
    let path = if base_path.is_empty() {
        format!("/{request_path}")
    } else if request_path.is_empty() {
        format!("{base_path}/")
    } else {
        format!("{base_path}/{request_path}")
    };
    url.set_path(&path);
    url.set_query((!request.query_string.is_empty()).then_some(&request.query_string));
    Ok(url)
}

fn response_headers(headers: &reqwest::header::HeaderMap) -> HeaderMap {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect()
}

fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}
