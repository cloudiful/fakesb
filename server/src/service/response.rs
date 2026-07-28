use serde_json::Value;

use crate::domain::{BodyFormat, HeaderMap, MockResponse};

pub fn body_format(content_type: Option<&str>) -> BodyFormat {
    let content_type = content_type
        .and_then(|value| value.split(';').next())
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if content_type == "application/json" || content_type.ends_with("+json") {
        BodyFormat::Json
    } else if content_type == "application/xml"
        || content_type == "text/xml"
        || content_type.ends_with("+xml")
    {
        BodyFormat::Xml
    } else {
        BodyFormat::Text
    }
}

pub fn normalize_body(raw_body: &str, content_type: &str) -> Option<Value> {
    match body_format(Some(content_type)) {
        BodyFormat::Json => serde_json::from_str(raw_body).ok(),
        BodyFormat::Xml => xml::convert::to_json(raw_body).ok(),
        BodyFormat::Text => None,
    }
}

pub fn from_body(
    status_code: u16,
    content_type: String,
    headers: HeaderMap,
    raw_body: String,
) -> MockResponse {
    let normalized_body = normalize_body(&raw_body, &content_type);
    MockResponse {
        status_code,
        content_type,
        headers,
        raw_body,
        normalized_body,
    }
}
