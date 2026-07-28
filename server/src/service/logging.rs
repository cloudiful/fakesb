use crate::domain::{MockRequest, MockResponse, RuleAction, SnapshotKind};
use crate::repositories;
use crate::service::{AppServices, ServiceError};

pub(super) async fn record_success(
    services: &AppServices,
    request: &MockRequest,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    action: RuleAction,
    response: &MockResponse,
    latency_ms: i64,
) -> Result<(), ServiceError> {
    let log_id = repositories::logs::insert(
        services.pool(),
        request,
        rule_id,
        target_id,
        Some(action),
        Some(response),
        Some(latency_ms),
        None,
    )
    .await?;

    if let Err(error) = insert_snapshots(services, log_id, request, response).await {
        let message = error.to_string();
        let _ = repositories::logs::mark_error(services.pool(), log_id, &message).await;
        return Err(error);
    }
    Ok(())
}

pub(super) async fn record_failure(
    services: &AppServices,
    request: &MockRequest,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    action: Option<RuleAction>,
    latency_ms: i64,
    error: &ServiceError,
) {
    let Ok(log_id) = repositories::logs::insert(
        services.pool(),
        request,
        rule_id,
        target_id,
        action,
        None,
        Some(latency_ms),
        Some(&error.to_string()),
    )
    .await
    else {
        return;
    };

    let _ = repositories::logs::insert_snapshot(
        services.pool(),
        log_id,
        SnapshotKind::Request,
        &request.raw_body,
        request
            .normalized_body
            .as_ref()
            .unwrap_or(&serde_json::Value::Null),
    )
    .await;
}

async fn insert_snapshots(
    services: &AppServices,
    log_id: i64,
    request: &MockRequest,
    response: &MockResponse,
) -> Result<(), ServiceError> {
    repositories::logs::insert_snapshot(
        services.pool(),
        log_id,
        SnapshotKind::Request,
        &request.raw_body,
        request
            .normalized_body
            .as_ref()
            .unwrap_or(&serde_json::Value::Null),
    )
    .await?;
    repositories::logs::insert_snapshot(
        services.pool(),
        log_id,
        SnapshotKind::Response,
        &response.raw_body,
        response
            .normalized_body
            .as_ref()
            .unwrap_or(&serde_json::Value::Null),
    )
    .await?;
    Ok(())
}
