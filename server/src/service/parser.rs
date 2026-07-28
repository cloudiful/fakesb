use std::collections::{BTreeMap, HashMap};

use actix_web::http::header::{CONTENT_TYPE, HeaderMap};
use serde_json::Value;

use crate::domain::{BodyFormat, HeaderMap as RequestHeaders, MockRequest};
use crate::service::{ServiceError, response};

pub fn parse_request(
    method: &str,
    uri: &str,
    headers: &HeaderMap,
    raw_body: &[u8],
) -> Result<MockRequest, ServiceError> {
    let (path, query_string) = uri.split_once('?').unwrap_or((uri, ""));
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body_format = response::body_format(content_type.as_deref());
    let raw_body = String::from_utf8(raw_body.to_vec())
        .map_err(|error| ServiceError::Parse(format!("request body must be UTF-8: {error}")))?;
    let normalized_body = match body_format {
        BodyFormat::Json => serde_json::from_str::<Value>(&raw_body)
            .map(Some)
            .map_err(|error| ServiceError::Parse(format!("invalid JSON request body: {error}")))?,
        BodyFormat::Xml => xml::convert::to_json(&raw_body)
            .map(Some)
            .map_err(|error| ServiceError::Parse(format!("invalid XML request body: {error}")))?,
        BodyFormat::Text => None,
    };

    Ok(MockRequest {
        method: method.to_ascii_uppercase(),
        path: if path.is_empty() {
            "/".into()
        } else {
            path.into()
        },
        query: parse_query(query_string),
        query_string: query_string.into(),
        headers: parse_headers(headers),
        content_type,
        body_format,
        raw_body,
        normalized_body,
    })
}

fn parse_headers(headers: &HeaderMap) -> RequestHeaders {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_ascii_lowercase(), value.to_string()))
        })
        .collect()
}

fn parse_query(query: &str) -> BTreeMap<String, Vec<String>> {
    let mut values: HashMap<String, Vec<String>> = HashMap::new();
    for (name, value) in url::form_urlencoded::parse(query.as_bytes()) {
        values
            .entry(name.into_owned())
            .or_default()
            .push(value.into_owned());
    }
    values.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use actix_web::http::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

    use super::parse_request;
    use crate::domain::BodyFormat;

    #[test]
    fn parses_json_request_metadata_and_body() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let request = parse_request("post", "/orders?state=ready", &headers, br#"{"id":7}"#)
            .expect("request should parse");

        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/orders");
        assert_eq!(request.query["state"], ["ready"]);
        assert_eq!(request.body_format, BodyFormat::Json);
        assert_eq!(request.body_value().unwrap()["id"], 7);
    }

    #[test]
    fn parses_xml_without_private_field_assumptions() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
        let request = parse_request("POST", "/orders", &headers, b"<order><id>7</id></order>")
            .expect("request should parse");

        assert_eq!(request.body_format, BodyFormat::Xml);
        assert_eq!(request.body_value().unwrap()["id"], "7");
    }

    #[test]
    fn rejects_invalid_json() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        assert!(parse_request("POST", "/", &headers, b"{").is_err());
    }
}
