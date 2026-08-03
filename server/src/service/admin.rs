use chrono::{DateTime, Utc};

use crate::domain::{
    Page, PaginationParams, RequestLog, ResponseTemplate, Rule, RuleAction, RuleMatcher, Target,
};
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
        let mut transaction = self.pool().begin().await?;
        let id = repositories::rules::insert(
            &mut *transaction,
            input.matcher,
            input.target_id,
            input.action,
            input.response_template_id,
            input.delay_ms,
            input.sequence_mode,
            input.priority,
            input.enabled,
            input.note,
        )
        .await?;
        if input.sequence_mode {
            replace_steps(&mut transaction, id, &input.sequence_steps).await?;
        }
        transaction.commit().await?;
        Ok(id)
    }

    pub async fn update_rule(
        &self,
        id: i64,
        input: RuleInput<'_>,
    ) -> Result<Option<i64>, ServiceError> {
        self.validate_rule(&input).await?;
        let mut transaction = self.pool().begin().await?;
        let updated = repositories::rules::update(
            &mut *transaction,
            id,
            input.matcher,
            input.target_id,
            input.action,
            input.response_template_id,
            input.delay_ms,
            input.sequence_mode,
            input.priority,
            input.enabled,
            input.note,
        )
        .await?;
        if let Some(id) = updated {
            repositories::sequences::reset_count(&mut *transaction, id).await?;
            if input.sequence_mode {
                replace_steps(&mut transaction, id, &input.sequence_steps).await?;
            } else {
                repositories::sequences::replace(&mut transaction, id, &[]).await?;
            }
        }
        transaction.commit().await?;
        Ok(updated)
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
        status_code: u16,
        headers: &serde_json::Value,
        enabled: bool,
        note: Option<&str>,
    ) -> Result<i64, ServiceError> {
        validate_template(
            name,
            content_type,
            raw_template,
            format,
            status_code,
            headers,
        )?;
        repositories::templates::insert(
            self.pool(),
            name,
            content_type,
            raw_template,
            format,
            status_code,
            headers,
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
        status_code: u16,
        headers: &serde_json::Value,
        enabled: bool,
        note: Option<&str>,
    ) -> Result<Option<i64>, ServiceError> {
        validate_template(
            name,
            content_type,
            raw_template,
            format,
            status_code,
            headers,
        )?;
        repositories::templates::update(
            self.pool(),
            id,
            name,
            content_type,
            raw_template,
            format,
            status_code,
            headers,
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
            query.method.as_deref(),
            query.path.as_deref(),
            query.action.as_deref(),
            query.status_code,
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

    pub async fn delete_log(&self, id: i64) -> Result<bool, ServiceError> {
        repositories::logs::delete(self.pool(), id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn purge_logs(&self, query: LogQuery) -> Result<i64, ServiceError> {
        repositories::logs::purge(
            self.pool(),
            query.method.as_deref(),
            query.path.as_deref(),
            query.action.as_deref(),
            query.status_code,
            query.start_time,
            query.end_time,
        )
        .await
        .map_err(ServiceError::from)
    }

    pub async fn delete_target(&self, id: i64) -> Result<bool, ServiceError> {
        if repositories::rules::count_by_target(self.pool(), id).await? > 0 {
            return Err(ServiceError::Conflict(
                "target is referenced by one or more rules".into(),
            ));
        }
        repositories::targets::delete(self.pool(), id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn delete_rule(&self, id: i64) -> Result<bool, ServiceError> {
        repositories::rules::delete(self.pool(), id)
            .await
            .map_err(ServiceError::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn test_rule(
        &self,
        method: &str,
        path: &str,
        query: &std::collections::BTreeMap<String, Vec<String>>,
        headers: &crate::domain::HeaderMap,
        content_type: Option<&str>,
        body: &str,
    ) -> Result<crate::domain::RuleTestResult, ServiceError> {
        let request = super::from_parts(
            method,
            path,
            query.clone(),
            &super::query_string(query),
            headers.clone(),
            content_type.map(ToOwned::to_owned),
            body,
        )?;
        let Some(rule) = super::dispatch::find_rule(self, &request).await? else {
            return Ok(crate::domain::RuleTestResult {
                matched: false,
                rule_id: None,
                action: None,
                priority: None,
                target_id: None,
                target_name: None,
                template_id: None,
                rendered: None,
            });
        };
        let target_id = rule.target_id.or(self.default_target_id());
        let target_name = match target_id {
            Some(id) => repositories::targets::find_enabled(self.pool(), id)
                .await?
                .map(|target| target.name),
            None => None,
        };
        let template_id = super::dispatch::preview_template_id(self, &rule).await?;
        let rendered = match rule.action {
            RuleAction::Static => {
                let response = super::mock::render(self, &request, template_id).await?;
                Some(crate::domain::RenderedPreview {
                    status_code: response.status_code,
                    content_type: response.content_type,
                    headers: crate::domain::headers_to_json(&response.headers),
                    raw_body: response.raw_body,
                })
            }
            RuleAction::Proxy => None,
        };
        Ok(crate::domain::RuleTestResult {
            matched: true,
            rule_id: Some(rule.id),
            action: Some(rule.action),
            priority: Some(rule.priority),
            target_id,
            target_name,
            template_id,
            rendered,
        })
    }

    pub async fn delete_template(&self, id: i64) -> Result<bool, ServiceError> {
        if repositories::rules::count_by_template(self.pool(), id).await? > 0 {
            return Err(ServiceError::Conflict(
                "template is referenced by one or more rules".into(),
            ));
        }
        if repositories::sequences::count_by_template(self.pool(), id).await? > 0 {
            return Err(ServiceError::Conflict(
                "template is referenced by one or more sequence steps".into(),
            ));
        }
        repositories::templates::delete(self.pool(), id)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn export_config(&self) -> Result<crate::domain::ExportBundle, ServiceError> {
        Ok(crate::domain::ExportBundle {
            targets: repositories::targets::list_all(self.pool()).await?,
            templates: repositories::templates::list_all(self.pool()).await?,
            rules: repositories::rules::list_all(self.pool()).await?,
        })
    }

    pub async fn import_config(
        &self,
        bundle: &crate::domain::ExportBundle,
    ) -> Result<crate::domain::ImportSummary, ServiceError> {
        let existing_rules = repositories::rules::list_all(self.pool()).await?;
        let mut imported_rule_ids = std::collections::HashMap::new();
        for rule in existing_rules {
            imported_rule_ids
                .entry(rule_import_identity(&rule))
                .or_insert(rule.id);
        }
        let mut transaction = self.pool().begin().await?;
        let mut summary = crate::domain::ImportSummary::default();

        let mut target_ids: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        for target in &bundle.targets {
            let id = repositories::targets::upsert(
                &mut *transaction,
                &target.name,
                &target.base_url,
                target.enabled,
                target.timeout_ms,
                target.note.as_deref(),
            )
            .await?;
            target_ids.insert(target.id, id);
            summary.targets_imported += 1;
        }

        let mut template_ids: std::collections::HashMap<i64, i64> =
            std::collections::HashMap::new();
        for template in &bundle.templates {
            let id = repositories::templates::upsert(
                &mut *transaction,
                &template.name,
                &template.content_type,
                &template.raw_template,
                &template.format,
                template.status_code,
                &template.headers,
                template.enabled,
                template.note.as_deref(),
            )
            .await?;
            template_ids.insert(template.id, id);
            summary.templates_imported += 1;
        }

        for rule in &bundle.rules {
            let target_id = rule.target_id.map(|id| *target_ids.get(&id).unwrap_or(&id));
            let template_id = rule
                .response_template_id
                .map(|id| *template_ids.get(&id).unwrap_or(&id));
            if rule.target_id.is_some() && !target_ids.contains_key(&rule.target_id.unwrap()) {
                summary.warnings.push(format!(
                    "rule #{} references a target not in the bundle (kept original id)",
                    rule.id
                ));
            }
            if rule.response_template_id.is_some()
                && !template_ids.contains_key(&rule.response_template_id.unwrap())
            {
                summary.warnings.push(format!(
                    "rule #{} references a template not in the bundle (kept original id)",
                    rule.id
                ));
            }
            let steps: Vec<crate::service::SequenceStepInput> = rule
                .sequence_steps
                .iter()
                .map(|step| {
                    let template_id = *template_ids.get(&step.template_id).unwrap_or(&step.template_id);
                    if !template_ids.contains_key(&step.template_id) {
                        summary.warnings.push(format!(
                            "rule #{} sequence step references a template not in the bundle (kept original id)",
                            rule.id
                        ));
                    }
                    crate::service::SequenceStepInput { template_id }
                })
                .collect();
            let identity = rule_import_identity(rule);
            let id = if let Some(id) = imported_rule_ids.get(&identity).copied() {
                repositories::rules::update(
                    &mut *transaction,
                    id,
                    &rule.matcher,
                    target_id,
                    rule.action,
                    template_id,
                    rule.delay_ms,
                    rule.sequence_mode,
                    rule.priority,
                    rule.enabled,
                    rule.note.as_deref(),
                )
                .await?
                .ok_or_else(|| sqlx::Error::Protocol("import rule disappeared".into()))?
            } else {
                let id = repositories::rules::insert(
                    &mut *transaction,
                    &rule.matcher,
                    target_id,
                    rule.action,
                    template_id,
                    rule.delay_ms,
                    rule.sequence_mode,
                    rule.priority,
                    rule.enabled,
                    rule.note.as_deref(),
                )
                .await?;
                imported_rule_ids.insert(identity, id);
                id
            };
            repositories::sequences::reset_count(&mut *transaction, id).await?;
            let steps: Vec<(i32, i64)> = steps
                .iter()
                .enumerate()
                .map(|(index, step)| (index as i32, step.template_id))
                .collect();
            repositories::sequences::replace(
                &mut transaction,
                id,
                if rule.sequence_mode { &steps } else { &[] },
            )
            .await?;
            summary.rules_imported += 1;
        }

        transaction.commit().await?;
        Ok(summary)
    }

    async fn validate_rule(&self, input: &RuleInput<'_>) -> Result<(), ServiceError> {
        crate::service::matching::validate(input.matcher)?;
        if input.action == RuleAction::Proxy && input.target_id.is_none() {
            return Err(ServiceError::Validation(
                "proxy rules require a target".into(),
            ));
        }
        if input.action == RuleAction::Static
            && !input.sequence_mode
            && input.response_template_id.is_none()
        {
            return Err(ServiceError::Validation(
                "static rules require a response template".into(),
            ));
        }
        if input.delay_ms < 0 {
            return Err(ServiceError::Validation(
                "delay_ms must not be negative".into(),
            ));
        }
        if input.sequence_mode {
            if input.action != RuleAction::Static {
                return Err(ServiceError::Validation(
                    "sequence rules must use the static action".into(),
                ));
            }
            if input.sequence_steps.is_empty() {
                return Err(ServiceError::Validation(
                    "sequence rules require at least one step".into(),
                ));
            }
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
        for step in &input.sequence_steps {
            if repositories::templates::find_enabled(self.pool(), step.template_id)
                .await?
                .is_none()
            {
                return Err(ServiceError::Validation(format!(
                    "sequence step template {} is missing or disabled",
                    step.template_id
                )));
            }
        }
        Ok(())
    }
}

fn rule_import_identity(rule: &Rule) -> String {
    let matcher = serde_json::to_string(&rule.matcher).unwrap_or_default();
    format!("{}\n{}\n{matcher}", rule.action.as_str(), rule.priority)
}

async fn replace_steps(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    rule_id: i64,
    steps: &[SequenceStepInput],
) -> Result<(), sqlx::Error> {
    let steps: Vec<(i32, i64)> = steps
        .iter()
        .enumerate()
        .map(|(index, step)| (index as i32, step.template_id))
        .collect();
    repositories::sequences::replace(&mut *transaction, rule_id, &steps).await
}

#[derive(Debug, Clone, Copy)]
pub struct SequenceStepInput {
    pub template_id: i64,
}

#[derive(Debug, Clone)]
pub struct RuleInput<'a> {
    pub matcher: &'a RuleMatcher,
    pub target_id: Option<i64>,
    pub action: RuleAction,
    pub response_template_id: Option<i64>,
    pub delay_ms: i32,
    pub sequence_mode: bool,
    pub sequence_steps: Vec<SequenceStepInput>,
    pub priority: i32,
    pub enabled: bool,
    pub note: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct LogQuery {
    pub page: PaginationParams,
    pub method: Option<String>,
    pub path: Option<String>,
    pub action: Option<String>,
    pub status_code: Option<i32>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

fn validate_target(name: &str, base_url: &str, timeout_ms: i32) -> Result<(), ServiceError> {
    if name.trim().is_empty() {
        return Err(ServiceError::Validation("target name is required".into()));
    }
    let url = reqwest::Url::parse(base_url)
        .map_err(|_| ServiceError::Validation("base_url must be a valid URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ServiceError::Validation(
            "base_url must use http or https and include a host".into(),
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
    status_code: u16,
    headers: &serde_json::Value,
) -> Result<(), ServiceError> {
    if name.trim().is_empty() || content_type.trim().is_empty() || raw_template.trim().is_empty() {
        return Err(ServiceError::Validation(
            "template name, content type and body are required".into(),
        ));
    }
    if !matches!(format, "json" | "xml" | "text") {
        return Err(ServiceError::Validation(
            "template format must be json, xml, or text".into(),
        ));
    }
    if !(100..=599).contains(&status_code) {
        return Err(ServiceError::Validation(
            "template status code must be between 100 and 599".into(),
        ));
    }
    crate::domain::json_to_headers(headers).map_err(ServiceError::Validation)?;
    Ok(())
}
