use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
    Passthrough,
    Mock,
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

impl RuleMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passthrough => "passthrough",
            Self::Mock => "mock",
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
    pub enabled: bool,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Rule {
    pub id: i64,
    pub service_code: String,
    pub message_type: String,
    pub message_code: String,
    pub target_id: Option<i64>,
    pub mode: RuleMode,
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
    pub mode: Option<RuleMode>,
    pub service_code: String,
    pub message_type: String,
    pub message_code: String,
    pub http_status_code: Option<String>,
    pub ret_code: Option<String>,
    pub ret_msg: Option<String>,
    pub latency_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageSnapshot {
    pub id: i64,
    pub kind: SnapshotKind,
    pub raw_body: String,
    pub normalized_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogDetail {
    #[serde(flatten)]
    pub log: RequestLog,
    pub snapshots: Vec<MessageSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedEsbMessage {
    pub service_code: String,
    pub message_type: String,
    pub message_code: String,
    pub normalized_json: serde_json::Value,
    pub raw_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsbResponse {
    pub status_code: u16,
    pub content_type: String,
    pub raw_body: String,
    pub normalized_json: Option<serde_json::Value>,
    pub ret_code: Option<String>,
    pub ret_msg: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::PaginationParams;

    #[test]
    fn pagination_is_bounded() {
        assert_eq!(PaginationParams::new(Some(-10), Some(0)).offset, 0);
        assert_eq!(PaginationParams::new(Some(-10), Some(0)).limit, 1);
        assert_eq!(PaginationParams::new(Some(4), Some(500)).limit, 100);
    }
}
