use std::time::Instant;

use crate::domain::{MockRequest, MockResponse, Rule, RuleAction};
use crate::repositories;
use crate::service::logging;
use crate::service::matching;
use crate::service::mock;
use crate::service::passthrough;
use crate::service::{AppServices, ServiceError};

pub async fn dispatch(
    services: &AppServices,
    request: &MockRequest,
) -> Result<MockResponse, ServiceError> {
    let started = Instant::now();
    let rule = match find_rule(services, request).await? {
        Some(rule) => rule,
        None => {
            let error = ServiceError::NoMatch;
            logging::record_failure(
                services,
                request,
                None,
                None,
                None,
                elapsed_ms(started),
                &error,
            )
            .await;
            return Err(error);
        }
    };
    let target_id = rule.target_id.or(services.default_target_id());
    let result = match rule.action {
        RuleAction::Proxy => passthrough::execute(services, request, target_id).await,
        RuleAction::Static => mock::render(services, request, rule.response_template_id).await,
    };

    match result {
        Ok(response) => {
            logging::record_success(
                services,
                request,
                Some(rule.id),
                target_id,
                rule.action,
                &response,
                elapsed_ms(started),
            )
            .await?;
            Ok(response)
        }
        Err(error) => {
            logging::record_failure(
                services,
                request,
                Some(rule.id),
                target_id,
                Some(rule.action),
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
    request: &MockRequest,
) -> Result<Option<Rule>, ServiceError> {
    let mut rules = repositories::rules::list_enabled(services.pool()).await?;
    sort_rules(&mut rules);
    for rule in rules {
        if matching::matches(&rule.matcher, request)? {
            return Ok(Some(rule));
        }
    }
    Ok(None)
}

fn sort_rules(rules: &mut [Rule]) {
    rules.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis() as i64
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::sort_rules;
    use crate::domain::{Rule, RuleAction, RuleMatcher};

    fn rule(id: i64, priority: i32) -> Rule {
        let now = Utc::now();
        Rule {
            id,
            matcher: RuleMatcher::default(),
            target_id: None,
            action: RuleAction::Static,
            response_template_id: None,
            priority,
            enabled: true,
            note: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn orders_rules_by_priority_then_id() {
        let mut rules = vec![rule(20, 0), rule(9, 10), rule(3, 10), rule(1, -1)];

        sort_rules(&mut rules);

        assert_eq!(
            rules.into_iter().map(|rule| rule.id).collect::<Vec<_>>(),
            vec![3, 9, 20, 1]
        );
    }
}
