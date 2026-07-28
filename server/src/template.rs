use chrono::Utc;
use regex::Regex;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::ParsedEsbMessage;
use crate::service::ServiceError;

pub fn render(raw_template: &str, request: &ParsedEsbMessage) -> Result<String, ServiceError> {
    let re = Regex::new(r"\{\{\s*(?P<expr>.*?)\s*\}\}")
        .map_err(|err| ServiceError::Template(err.to_string()))?;

    let mut rendered = String::with_capacity(raw_template.len());
    let mut last = 0;

    for captures in re.captures_iter(raw_template) {
        let matched = captures.get(0).expect("capture 0 exists");
        rendered.push_str(&raw_template[last..matched.start()]);

        let expr = captures
            .name("expr")
            .map(|m| m.as_str())
            .unwrap_or_default()
            .trim();
        rendered.push_str(&resolve(expr, request)?);
        last = matched.end();
    }

    rendered.push_str(&raw_template[last..]);
    Ok(rendered)
}

fn resolve(expr: &str, request: &ParsedEsbMessage) -> Result<String, ServiceError> {
    if expr == "now" {
        return Ok(Utc::now().to_rfc3339());
    }
    if expr == "timestamp_ms" {
        return Ok(Utc::now().timestamp_millis().to_string());
    }
    if let Some(format) = expr.strip_prefix("date:") {
        return Ok(Utc::now().format(format).to_string());
    }
    if expr == "uuid" {
        return Ok(Uuid::new_v4().to_string());
    }
    if let Some(len) = expr.strip_prefix("rand_numeric:") {
        let len = len
            .parse::<usize>()
            .map_err(|_| ServiceError::Template(format!("invalid rand_numeric length: {expr}")))?;
        return Ok(random_numeric(len));
    }
    if let Some(path_expr) = expr.strip_prefix("req.") {
        return resolve_req_value(path_expr, request);
    }

    Err(ServiceError::Template(format!(
        "unsupported template expression: {expr}"
    )))
}

fn resolve_req_value(path_expr: &str, request: &ParsedEsbMessage) -> Result<String, ServiceError> {
    let (path, default_value) = if let Some((path, fallback)) = path_expr.split_once("|default:") {
        (path, Some(strip_quoted(fallback.trim())))
    } else {
        (path_expr, None)
    };

    let mut current = &request.normalized_json;
    for segment in path.split('.') {
        match current {
            Value::Object(map) => {
                current = match map.get(segment) {
                    Some(value) => value,
                    None => return default_or_error(default_value.clone(), path),
                };
            }
            Value::Array(items) => {
                let index = segment.parse::<usize>().map_err(|_| {
                    ServiceError::Template(format!(
                        "array segment must be numeric for req.{path}: {segment}"
                    ))
                })?;
                current = match items.get(index) {
                    Some(value) => value,
                    None => return default_or_error(default_value.clone(), path),
                };
            }
            _ => {
                return default_or_error(default_value, path);
            }
        }
    }

    match current {
        Value::Null => default_or_error(default_value, path),
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        other => Ok(other.to_string()),
    }
}

fn default_or_error(default_value: Option<String>, path: &str) -> Result<String, ServiceError> {
    default_value.ok_or_else(|| {
        ServiceError::Template(format!(
            "request field not found and no default provided: req.{path}"
        ))
    })
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
    use super::render;
    use crate::domain::ParsedEsbMessage;

    fn request() -> ParsedEsbMessage {
        ParsedEsbMessage {
            service_code: "SVC".into(),
            message_type: "TYPE".into(),
            message_code: "CODE".into(),
            normalized_json: serde_json::json!({
                "body": { "customer": { "name": "Ada" }, "items": [{ "id": 7 }] }
            }),
            raw_body: "<request />".into(),
        }
    }

    #[test]
    fn renders_request_values_and_defaults() {
        let result = render(
            "<response><name>{{ req.body.customer.name }}</name><missing>{{ req.body.missing|default:\"N/A\" }}</missing></response>",
            &request(),
        )
        .expect("template should render");
        assert_eq!(
            result,
            "<response><name>Ada</name><missing>N/A</missing></response>"
        );
    }

    #[test]
    fn renders_array_indexes() {
        let result =
            render("{{ req.body.items.0.id }}", &request()).expect("template should render");
        assert_eq!(result, "7");
    }
}
