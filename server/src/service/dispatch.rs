use std::time::Instant;

use crate::domain::{ParsedEsbMessage, Rule, RuleMode};
use crate::repositories;
use crate::service::logging;
use crate::service::mock;
use crate::service::parser::parse_request;
use crate::service::passthrough;
use crate::service::{AppServices, ServiceError};

pub async fn dispatch(
    services: &AppServices,
    raw_body: &str,
) -> Result<crate::domain::EsbResponse, ServiceError> {
    let request = parse_request(raw_body)?;
    let started = Instant::now();
    let rule = match find_rule(services, &request).await {
        Ok(rule) => rule,
        Err(error) => {
            logging::record_failure(
                services,
                &request,
                None,
                services.default_target_id(),
                None,
                elapsed_ms(started),
                &error,
            )
            .await;
            return Err(error);
        }
    };
    let target_id = rule.target_id.or(services.default_target_id());

    let result = match rule.mode {
        RuleMode::Passthrough => passthrough::execute(services, &request, target_id).await,
        RuleMode::Mock => mock::render(services, &request, rule.response_template_id).await,
    };

    match result {
        Ok(response) => {
            logging::record_success(
                services,
                &request,
                (rule.id != 0).then_some(rule.id),
                target_id,
                rule.mode,
                &response,
                elapsed_ms(started),
            )
            .await?;
            Ok(response)
        }
        Err(error) => {
            logging::record_failure(
                services,
                &request,
                (rule.id != 0).then_some(rule.id),
                target_id,
                Some(rule.mode),
                elapsed_ms(started),
                &error,
            )
            .await;
            Err(error)
        }
    }
}

async fn find_rule(
    services: &AppServices,
    request: &ParsedEsbMessage,
) -> Result<Rule, ServiceError> {
    if let Some(rule) = repositories::rules::find_match(
        services.pool(),
        &request.service_code,
        &request.message_type,
        &request.message_code,
    )
    .await?
    {
        return Ok(rule);
    }

    Ok(Rule {
        id: 0,
        service_code: request.service_code.clone(),
        message_type: request.message_type.clone(),
        message_code: request.message_code.clone(),
        target_id: None,
        mode: RuleMode::Passthrough,
        response_template_id: None,
        priority: 0,
        enabled: true,
        note: Some("default passthrough".into()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis() as i64
}
