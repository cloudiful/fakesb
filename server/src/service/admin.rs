use chrono::{DateTime, Utc};

use crate::domain::{Page, PaginationParams, RequestLog, ResponseTemplate, Rule, RuleMode, Target};
use crate::repositories;

use super::{AppServices, ServiceError};

impl AppServices {
    pub async fn list_targets(&self, page: PaginationParams) -> Result<Page<Target>, ServiceError> {
        repositories::targets::list(self.pool(), page)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn create_target(
        &self,
        name: &str,
        base_url: &str,
        enabled: bool,
        timeout_ms: i32,
        note: Option<&str>,
    ) -> Result<i64, ServiceError> {
        validate_target(name, base_url, timeout_ms)?;
        repositories::targets::insert(self.pool(), name, base_url, enabled, timeout_ms, note)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn update_target(
        &self,
        id: i64,
        name: &str,
        base_url: &str,
        enabled: bool,
        timeout_ms: i32,
        note: Option<&str>,
    ) -> Result<Option<i64>, ServiceError> {
        validate_target(name, base_url, timeout_ms)?;
        repositories::targets::update(self.pool(), id, name, base_url, enabled, timeout_ms, note)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn list_rules(&self, page: PaginationParams) -> Result<Page<Rule>, ServiceError> {
        repositories::rules::list(self.pool(), page)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn create_rule(&self, input: RuleInput<'_>) -> Result<i64, ServiceError> {
        self.validate_rule(&input).await?;
        repositories::rules::insert(
            self.pool(),
            input.service_code,
            input.message_type,
            input.message_code,
            input.target_id,
            input.mode.as_str(),
            input.response_template_id,
            input.priority,
            input.enabled,
            input.note,
        )
        .await
        .map_err(ServiceError::from)
    }

    pub async fn update_rule(
        &self,
        id: i64,
        input: RuleInput<'_>,
    ) -> Result<Option<i64>, ServiceError> {
        self.validate_rule(&input).await?;
        repositories::rules::update(
            self.pool(),
            id,
            input.service_code,
            input.message_type,
            input.message_code,
            input.target_id,
            input.mode.as_str(),
            input.response_template_id,
            input.priority,
            input.enabled,
            input.note,
        )
        .await
        .map_err(ServiceError::from)
    }

    pub async fn list_templates(
        &self,
        page: PaginationParams,
    ) -> Result<Page<ResponseTemplate>, ServiceError> {
        repositories::templates::list(self.pool(), page)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn create_template(
        &self,
        name: &str,
        content_type: &str,
        raw_template: &str,
        format: &str,
        enabled: bool,
        note: Option<&str>,
    ) -> Result<i64, ServiceError> {
        validate_template(name, content_type, raw_template, format)?;
        repositories::templates::insert(
            self.pool(),
            name,
            content_type,
            raw_template,
            format,
            enabled,
            note,
        )
        .await
        .map_err(ServiceError::from)
    }

    pub async fn update_template(
        &self,
        id: i64,
        name: &str,
        content_type: &str,
        raw_template: &str,
        format: &str,
        enabled: bool,
        note: Option<&str>,
    ) -> Result<Option<i64>, ServiceError> {
        validate_template(name, content_type, raw_template, format)?;
        repositories::templates::update(
            self.pool(),
            id,
            name,
            content_type,
            raw_template,
            format,
            enabled,
            note,
        )
        .await
        .map_err(ServiceError::from)
    }

    pub async fn list_logs(&self, query: LogQuery) -> Result<Page<RequestLog>, ServiceError> {
        repositories::logs::list(
            self.pool(),
            query.page,
            query.service_code.as_deref(),
            query.message_type.as_deref(),
            query.message_code.as_deref(),
            query.mode.as_deref(),
            query.ret_code.as_deref(),
            query.start_time,
            query.end_time,
        )
        .await
        .map_err(ServiceError::from)
    }

    pub async fn log_detail(
        &self,
        id: i64,
    ) -> Result<Option<crate::domain::LogDetail>, ServiceError> {
        repositories::logs::detail(self.pool(), id)
            .await
            .map_err(ServiceError::from)
    }

    async fn validate_rule(&self, input: &RuleInput<'_>) -> Result<(), ServiceError> {
        if input.service_code.trim().is_empty()
            || input.message_type.trim().is_empty()
            || input.message_code.trim().is_empty()
        {
            return Err(ServiceError::Validation(
                "service and message identifiers are required".into(),
            ));
        }
        if matches!(input.mode, RuleMode::Mock) && input.response_template_id.is_none() {
            return Err(ServiceError::Validation(
                "mock rules require a response template".into(),
            ));
        }
        if let Some(target_id) = input.target_id {
            if repositories::targets::find_enabled(self.pool(), target_id)
                .await?
                .is_none()
            {
                return Err(ServiceError::Validation(
                    "target is missing or disabled".into(),
                ));
            }
        }
        if let Some(template_id) = input.response_template_id {
            if repositories::templates::find_enabled(self.pool(), template_id)
                .await?
                .is_none()
            {
                return Err(ServiceError::Validation(
                    "response template is missing or disabled".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RuleInput<'a> {
    pub service_code: &'a str,
    pub message_type: &'a str,
    pub message_code: &'a str,
    pub target_id: Option<i64>,
    pub mode: RuleMode,
    pub response_template_id: Option<i64>,
    pub priority: i32,
    pub enabled: bool,
    pub note: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct LogQuery {
    pub page: PaginationParams,
    pub service_code: Option<String>,
    pub message_type: Option<String>,
    pub message_code: Option<String>,
    pub mode: Option<String>,
    pub ret_code: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

fn validate_target(name: &str, base_url: &str, timeout_ms: i32) -> Result<(), ServiceError> {
    if name.trim().is_empty() {
        return Err(ServiceError::Validation("target name is required".into()));
    }
    let url = reqwest::Url::parse(base_url)
        .map_err(|_| ServiceError::Validation("base_url must be a valid URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ServiceError::Validation(
            "base_url must use http or https".into(),
        ));
    }
    if timeout_ms <= 0 {
        return Err(ServiceError::Validation(
            "timeout_ms must be positive".into(),
        ));
    }
    Ok(())
}

fn validate_template(
    name: &str,
    content_type: &str,
    raw_template: &str,
    format: &str,
) -> Result<(), ServiceError> {
    if name.trim().is_empty() || content_type.trim().is_empty() || raw_template.trim().is_empty() {
        return Err(ServiceError::Validation(
            "template name, content type and body are required".into(),
        ));
    }
    if format != "xml" {
        return Err(ServiceError::Validation(
            "only xml templates are supported".into(),
        ));
    }
    Ok(())
}
