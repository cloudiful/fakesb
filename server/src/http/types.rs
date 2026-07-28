use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::domain::{PaginationParams, RequestLog, ResponseTemplate, Rule, RuleMode, Target};

#[derive(Debug, Deserialize, IntoParams)]
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
pub struct RulePayload {
    pub service_code: String,
    pub message_type: String,
    pub message_code: String,
    pub target_id: Option<i64>,
    pub mode: RuleMode,
    pub response_template_id: Option<i64>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TemplatePayload {
    pub name: String,
    pub content_type: Option<String>,
    pub raw_template: String,
    pub format: Option<String>,
    pub enabled: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct LogQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub service_code: Option<String>,
    pub message_type: Option<String>,
    pub message_code: Option<String>,
    pub mode: Option<RuleMode>,
    pub ret_code: Option<String>,
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
            service_code: normalize(self.service_code),
            message_type: normalize(self.message_type),
            message_code: normalize(self.message_code),
            mode: self.mode.map(|value| value.as_str().to_string()),
            ret_code: normalize(self.ret_code),
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

fn normalize(value: Option<String>) -> Option<String> {
    value.and_then(|value| (!value.trim().is_empty()).then_some(value))
}
