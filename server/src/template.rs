use chrono::Utc;
use regex::Regex;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{MockRequest, MockResponse, value_at_path};
use crate::service::ServiceError;

pub fn render_for_format(
    raw_template: &str,
    format: &str,
    request: &MockRequest,
) -> Result<String, ServiceError> {
    render(raw_template, format, request, None)
}

pub fn render_for_response(
    raw_template: &str,
    format: &str,
    request: &MockRequest,
    response: &MockResponse,
) -> Result<String, ServiceError> {
    render(raw_template, format, request, Some(response))
}

fn render(
    raw_template: &str,
    format: &str,
    request: &MockRequest,
    response: Option<&MockResponse>,
) -> Result<String, ServiceError> {
    let re = Regex::new(r"\{\{\s*(?P<expr>.*?)\s*\}\}")
        .map_err(|err| ServiceError::Template(err.to_string()))?;

    let mut rendered = String::with_capacity(raw_template.len());
    let mut last = 0;
    for captures in re.captures_iter(raw_template) {
        let matched = captures.get(0).expect("capture 0 exists");
        rendered.push_str(&raw_template[last..matched.start()]);
        let expr = captures
            .name("expr")
            .map(|value| value.as_str())
            .unwrap_or_default()
            .trim();
        let value = resolve_value(expr, request, response)?;
        rendered.push_str(&format_value(raw_template, matched.start(), format, &value));
        last = matched.end();
    }
    rendered.push_str(&raw_template[last..]);
    Ok(rendered)
}

fn resolve_value(
    expr: &str,
    request: &MockRequest,
    response: Option<&MockResponse>,
) -> Result<Value, ServiceError> {
    if expr == "now" {
        return Ok(Value::String(Utc::now().to_rfc3339()));
    }
    if expr == "timestamp_ms" {
        return Ok(Value::Number(Utc::now().timestamp_millis().into()));
    }
    if let Some(format) = expr.strip_prefix("date:") {
        return Ok(Value::String(Utc::now().format(format).to_string()));
    }
    if expr == "uuid" {
        return Ok(Value::String(Uuid::new_v4().to_string()));
    }
    if let Some(len) = expr.strip_prefix("rand_numeric:") {
        let len = len
            .parse::<usize>()
            .map_err(|_| ServiceError::Template(format!("invalid rand_numeric length: {expr}")))?;
        return Ok(Value::String(random_numeric(len)));
    }
    if let Some(path_expr) = expr.strip_prefix("req.") {
        return resolve_request_value(path_expr, request);
    }
    if let Some(path_expr) = expr.strip_prefix("resp.") {
        return resolve_response_value(path_expr, response);
    }
    Err(ServiceError::Template(format!(
        "unsupported template expression: {expr}"
    )))
}

fn resolve_request_value(path_expr: &str, request: &MockRequest) -> Result<Value, ServiceError> {
    let (path, default_value) = if let Some((path, fallback)) = path_expr.split_once("|default:") {
        (path, Some(strip_quoted(fallback.trim())))
    } else {
        (path_expr, None)
    };

    let value = if path == "method" {
        Some(Value::String(request.method.clone()))
    } else if path == "path" {
        Some(Value::String(request.path.clone()))
    } else if let Some(query_name) = path.strip_prefix("query.") {
        request
            .query_values(query_name)
            .and_then(|values| values.first())
            .cloned()
            .map(Value::String)
    } else if let Some(header_name) = path.strip_prefix("headers.") {
        request
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(header_name))
            .map(|(_, value)| Value::String(value.clone()))
    } else if let Some(body_path) = path.strip_prefix("body") {
        let body_path = body_path.trim_start_matches('.');
        if body_path.is_empty() {
            request.body_value().cloned()
        } else {
            request
                .body_value()
                .and_then(|body| value_at_path(body, body_path).cloned())
        }
    } else {
        None
    };

    match value {
        Some(Value::Null) | None => default_or_error(default_value, path),
        Some(value) => Ok(value),
    }
}

fn default_or_error(default_value: Option<String>, path: &str) -> Result<Value, ServiceError> {
    default_value
        .ok_or_else(|| {
            ServiceError::Template(format!(
                "request field not found and no default provided: req.{path}"
            ))
        })
        .map(Value::String)
}

fn resolve_response_value(
    path_expr: &str,
    response: Option<&MockResponse>,
) -> Result<Value, ServiceError> {
    let response = response.ok_or_else(|| {
        ServiceError::Template("resp.* cannot be used without an upstream response".into())
    })?;
    let (path, default_value) = if let Some((path, fallback)) = path_expr.split_once("|default:") {
        (path, Some(strip_quoted(fallback.trim())))
    } else {
        (path_expr, None)
    };

    let value = if path == "status" {
        Some(Value::Number(response.status_code.into()))
    } else if let Some(header_name) = path.strip_prefix("headers.") {
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(header_name))
            .map(|(_, value)| Value::String(value.clone()))
    } else if let Some(body_path) = path.strip_prefix("body") {
        let body_path = body_path.trim_start_matches('.');
        if body_path.is_empty() {
            response
                .normalized_body
                .clone()
                .or_else(|| Some(Value::String(response.raw_body.clone())))
        } else {
            response
                .normalized_body
                .as_ref()
                .and_then(|body| value_at_path(body, body_path).cloned())
        }
    } else {
        None
    };

    match value {
        Some(Value::Null) | None => default_or_error(default_value, path),
        Some(value) => Ok(value),
    }
}

fn format_value(template: &str, start: usize, format: &str, value: &Value) -> String {
    match format {
        "json" if in_json_string(template, start) => {
            let value = value_to_string(value);
            let serialized =
                serde_json::to_string(&value).expect("JSON string serialization cannot fail");
            serialized[1..serialized.len() - 1].to_string()
        }
        "json" => serde_json::to_string(value).expect("JSON value serialization cannot fail"),
        "xml" => xml_escape(&value_to_string(value)),
        _ => value_to_string(value),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

fn in_json_string(template: &str, end: usize) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    for byte in template.as_bytes().iter().take(end) {
        if escaped {
            escaped = false;
        } else if *byte == b'\\' && in_string {
            escaped = true;
        } else if *byte == b'"' {
            in_string = !in_string;
        }
    }
    in_string
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn strip_quoted(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
        .to_string()
}

fn random_numeric(len: usize) -> String {
    let seed = Uuid::new_v4().as_u128().to_string();
    seed.chars()
        .filter(|ch| ch.is_ascii_digit())
        .cycle()
        .take(len)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::Value;

    use super::render_for_format;
    use crate::domain::{BodyFormat, MockRequest};

    fn request() -> MockRequest {
        MockRequest {
            method: "POST".into(),
            path: "/orders".into(),
            query: BTreeMap::from([(String::from("source"), vec![String::from("test")])]),
            query_string: "source=test".into(),
            headers: BTreeMap::from([(String::from("x-request-id"), String::from("abc"))]),
            content_type: Some("application/json".into()),
            body_format: BodyFormat::Json,
            raw_body: r#"{"body":{"customer":{"name":"Ada"},"items":[{"id":7}]}}"#.into(),
            normalized_body: Some(serde_json::json!({
                "body": { "customer": { "name": "Ada" }, "items": [{ "id": 7 }] }
            })),
        }
    }

    #[test]
    fn renders_generic_request_values_and_defaults() {
        let result = render_for_format(
            "<response><name>{{ req.body.body.customer.name }}</name><path>{{ req.path }}</path><missing>{{ req.body.missing|default:\"N/A\" }}</missing></response>",
            "text",
            &request(),
        )
        .expect("template should render");
        assert_eq!(
            result,
            "<response><name>Ada</name><path>/orders</path><missing>N/A</missing></response>"
        );
    }

    #[test]
    fn renders_query_and_header_values() {
        let result = render_for_format(
            "{{ req.query.source }} {{ req.headers.X-Request-Id }}",
            "text",
            &request(),
        )
        .expect("template should render");
        assert_eq!(result, "test abc");
    }

    #[test]
    fn escapes_json_string_values_and_serializes_json_values() {
        let mut request = request();
        request.normalized_body = Some(serde_json::json!({
            "body": { "name": "Ada \"Lovelace\"", "id": 7 }
        }));

        let result = render_for_format(
            r#"{"name":"{{ req.body.body.name }}","id":{{ req.body.body.id }}}"#,
            "json",
            &request,
        )
        .expect("JSON template should render");

        assert_eq!(
            serde_json::from_str::<Value>(&result).unwrap(),
            serde_json::json!({"name": "Ada \"Lovelace\"", "id": 7})
        );
    }

    #[test]
    fn escapes_xml_values() {
        let mut request = request();
        request.normalized_body = Some(serde_json::json!({
            "body": { "name": "Ada & <Lovelace>" }
        }));

        let result = render_for_format("<name>{{ req.body.body.name }}</name>", "xml", &request)
            .expect("XML template should render");

        assert_eq!(result, "<name>Ada &amp; &lt;Lovelace&gt;</name>");
    }

    #[test]
    fn renders_upstream_response_values() {
        use super::render_for_response;
        use crate::domain::MockResponse;

        let response = MockResponse {
            status_code: 201,
            content_type: "application/json".into(),
            headers: BTreeMap::from([(String::from("x-correlation"), String::from("c1"))]),
            raw_body: r#"{"order":{"id":9}}"#.into(),
            normalized_body: Some(serde_json::json!({"order": {"id": 9}})),
        };
        let result = render_for_response(
            "{{ resp.status }} {{ resp.headers.X-Correlation }} {{ resp.body.order.id }}",
            "text",
            &request(),
            &response,
        )
        .expect("template should render");

        assert_eq!(result, "201 c1 9");
    }

    #[test]
    fn renders_raw_text_from_upstream_response() {
        use super::render_for_response;
        use crate::domain::MockResponse;

        let response = MockResponse {
            status_code: 200,
            content_type: "text/plain".into(),
            headers: BTreeMap::new(),
            raw_body: "upstream text".into(),
            normalized_body: None,
        };
        let result = render_for_response("{{ resp.body }}", "text", &request(), &response)
            .expect("text response should render");

        assert_eq!(result, "upstream text");
    }
}
