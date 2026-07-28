use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub type HeaderMap = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Proxy,
    Static,
}

impl RuleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proxy => "proxy",
            Self::Static => "static",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BodyFormat {
    Json,
    Xml,
    #[default]
    Text,
}

impl BodyFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Text => "text",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct StringMatcher {
    #[serde(default)]
    pub equal_to: Option<String>,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    pub matches: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct BodyMatcher {
    pub format: BodyFormat,
    #[serde(default)]
    pub equal_to: Option<String>,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    pub matches: Option<String>,
    #[serde(default)]
    pub json_equal_to: Option<Value>,
    #[serde(default)]
    pub fields: BTreeMap<String, StringMatcher>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct RuleMatcher {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub path_pattern: Option<String>,
    #[serde(default)]
    pub query: BTreeMap<String, StringMatcher>,
    #[serde(default)]
    pub headers: BTreeMap<String, StringMatcher>,
    #[serde(default)]
    pub body: Option<BodyMatcher>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    Request,
    Response,
}

impl SnapshotKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Response => "response",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Target {
    pub id: i64,
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
    pub timeout_ms: i32,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResponseTemplate {
    pub id: i64,
    pub name: String,
    pub content_type: String,
    pub raw_template: String,
    pub format: String,
    pub status_code: u16,
    pub headers: Value,
    pub enabled: bool,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Rule {
    pub id: i64,
    pub matcher: RuleMatcher,
    pub target_id: Option<i64>,
    pub action: RuleAction,
    pub response_template_id: Option<i64>,
    pub priority: i32,
    pub enabled: bool,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequestLog {
    pub id: i64,
    pub occurred_at: DateTime<Utc>,
    pub rule_id: Option<i64>,
    pub target_id: Option<i64>,
    pub action: Option<RuleAction>,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub content_type: Option<String>,
    pub body_format: BodyFormat,
    pub request_headers: Value,
    pub response_headers: Value,
    pub http_status_code: Option<u16>,
    pub latency_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageSnapshot {
    pub id: i64,
    pub kind: SnapshotKind,
    pub raw_body: String,
    pub normalized_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogDetail {
    #[serde(flatten)]
    pub log: RequestLog,
    pub snapshots: Vec<MessageSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRequest {
    pub method: String,
    pub path: String,
    pub query: BTreeMap<String, Vec<String>>,
    pub query_string: String,
    pub headers: HeaderMap,
    pub content_type: Option<String>,
    pub body_format: BodyFormat,
    pub raw_body: String,
    pub normalized_body: Option<Value>,
}

impl MockRequest {
    pub fn body_value(&self) -> Option<&Value> {
        self.normalized_body.as_ref()
    }

    pub fn query_values(&self, name: &str) -> Option<&[String]> {
        self.query.get(name).map(Vec::as_slice)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    pub status_code: u16,
    pub content_type: String,
    pub headers: HeaderMap,
    pub raw_body: String,
    pub normalized_body: Option<Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct PaginationParams {
    pub offset: i64,
    pub limit: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}

impl PaginationParams {
    pub fn new(offset: Option<i64>, limit: Option<i64>) -> Self {
        Self {
            offset: offset.unwrap_or(0).max(0),
            limit: limit.unwrap_or(20).clamp(1, 100),
        }
    }
}

pub fn value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.trim_start_matches("$.")
        .split('.')
        .filter(|segment| !segment.is_empty())
        .try_fold(value, |current, segment| match current {
            Value::Object(map) => map.get(segment),
            Value::Array(items) => segment
                .parse::<usize>()
                .ok()
                .and_then(|index| items.get(index)),
            _ => None,
        })
}

pub fn headers_to_json(headers: &HeaderMap) -> Value {
    serde_json::to_value(headers).unwrap_or_else(|_| Value::Object(Default::default()))
}

pub fn json_to_headers(value: &Value) -> Result<HeaderMap, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "headers must be a JSON object".to_string())?;
    let mut headers = BTreeMap::new();
    for (name, value) in object {
        let value = value
            .as_str()
            .ok_or_else(|| format!("header value must be a string: {name}"))?;
        headers.insert(name.clone(), value.to_string());
    }
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::{PaginationParams, value_at_path};

    #[test]
    fn pagination_is_bounded() {
        assert_eq!(PaginationParams::new(Some(-10), Some(0)).offset, 0);
        assert_eq!(PaginationParams::new(Some(-10), Some(0)).limit, 1);
        assert_eq!(PaginationParams::new(Some(4), Some(500)).limit, 100);
    }

    #[test]
    fn reads_nested_values_with_dot_paths() {
        let value = serde_json::json!({"body": {"items": [{"id": 7}]}});
        assert_eq!(value_at_path(&value, "body.items.0.id").unwrap(), 7);
    }
}
