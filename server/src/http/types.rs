use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::domain::{
    PaginationParams, RequestLog, ResponseTemplate, Rule, RuleAction, RuleMatcher, Target,
};
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PagingQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

impl PagingQuery {
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams::new(self.offset, self.limit)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TargetPayload {
    pub name: String,
    pub base_url: String,
    pub enabled: Option<bool>,
    pub timeout_ms: Option<i32>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SequenceStepPayload {
    pub template_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RulePayload {
    pub matcher: RuleMatcher,
    pub target_id: Option<i64>,
    pub action: RuleAction,
    pub response_template_id: Option<i64>,
    pub priority: Option<i32>,
    pub delay_ms: Option<i32>,
    pub sequence_mode: Option<bool>,
    #[serde(default)]
    pub sequence_steps: Vec<SequenceStepPayload>,
    pub enabled: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TemplatePayload {
    pub name: String,
    pub content_type: Option<String>,
    pub raw_template: String,
    pub format: Option<String>,
    pub status_code: Option<u16>,
    pub headers: Option<serde_json::Value>,
    pub enabled: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RuleTestPayload {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub query: std::collections::BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub headers: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LogQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub action: Option<RuleAction>,
    pub status_code: Option<i32>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl LogQuery {
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams::new(self.offset, self.limit)
    }

    pub fn into_service_query(self) -> crate::service::LogQuery {
        crate::service::LogQuery {
            page: self.pagination(),
            method: normalize(self.method),
            path: normalize(self.path),
            action: self.action.map(|value| value.as_str().to_string()),
            status_code: self.status_code,
            start_time: self.start_time,
            end_time: self.end_time,
        }
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct IdResponse {
    pub id: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TargetPage {
    pub items: Vec<Target>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct RulePage {
    pub items: Vec<Rule>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct TemplatePage {
    pub items: Vec<ResponseTemplate>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LogPage {
    pub items: Vec<RequestLog>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PurgeResponse {
    pub deleted: i64,
}

fn normalize(value: Option<String>) -> Option<String> {
    value.and_then(|value| (!value.trim().is_empty()).then_some(value))
}
