use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::domain::{
    BodyFormat, LogDetail, MessageSnapshot, Page, PaginationParams, RequestLog, RuleAction,
    SnapshotKind,
};

#[derive(Debug, FromRow)]
struct RequestLogRow {
    id: i64,
    occurred_at: DateTime<Utc>,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    action: Option<String>,
    method: String,
    path: String,
    query_string: Option<String>,
    content_type: Option<String>,
    body_format: String,
    request_headers: serde_json::Value,
    response_headers: serde_json::Value,
    http_status_code: Option<i32>,
    latency_ms: Option<i64>,
    error_message: Option<String>,
}

impl TryFrom<RequestLogRow> for RequestLog {
    type Error = String;

    fn try_from(row: RequestLogRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            occurred_at: row.occurred_at,
            rule_id: row.rule_id,
            target_id: row.target_id,
            action: row.action.map(|value| parse_action(&value)).transpose()?,
            method: row.method,
            path: row.path,
            query_string: row.query_string,
            content_type: row.content_type,
            body_format: parse_body_format(&row.body_format)?,
            request_headers: row.request_headers,
            response_headers: row.response_headers,
            http_status_code: row.http_status_code.map(|value| value as u16),
            latency_ms: row.latency_ms,
            error_message: row.error_message,
        })
    }
}

#[derive(Debug, FromRow)]
struct RequestLogListRow {
    total: i64,
    id: i64,
    occurred_at: DateTime<Utc>,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    action: Option<String>,
    method: String,
    path: String,
    query_string: Option<String>,
    content_type: Option<String>,
    body_format: String,
    request_headers: serde_json::Value,
    response_headers: serde_json::Value,
    http_status_code: Option<i32>,
    latency_ms: Option<i64>,
    error_message: Option<String>,
}

impl TryFrom<RequestLogListRow> for RequestLog {
    type Error = String;

    fn try_from(row: RequestLogListRow) -> Result<Self, Self::Error> {
        RequestLogRow {
            id: row.id,
            occurred_at: row.occurred_at,
            rule_id: row.rule_id,
            target_id: row.target_id,
            action: row.action,
            method: row.method,
            path: row.path,
            query_string: row.query_string,
            content_type: row.content_type,
            body_format: row.body_format,
            request_headers: row.request_headers,
            response_headers: row.response_headers,
            http_status_code: row.http_status_code,
            latency_ms: row.latency_ms,
            error_message: row.error_message,
        }
        .try_into()
    }
}

#[derive(Debug, FromRow)]
struct LogDetailRow {
    id: i64,
    occurred_at: DateTime<Utc>,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    action: Option<String>,
    method: String,
    path: String,
    query_string: Option<String>,
    content_type: Option<String>,
    body_format: String,
    request_headers: serde_json::Value,
    response_headers: serde_json::Value,
    http_status_code: Option<i32>,
    latency_ms: Option<i64>,
    error_message: Option<String>,
    snapshots: serde_json::Value,
}

pub async fn list(
    pool: &PgPool,
    page: PaginationParams,
    method: Option<&str>,
    path: Option<&str>,
    action: Option<&str>,
    status_code: Option<i32>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> Result<Page<RequestLog>, sqlx::Error> {
    let rows = sqlx::query_file_as!(
        RequestLogListRow,
        "sql/logs/list.sql",
        method,
        path,
        action,
        status_code,
        start_time,
        end_time,
        page.limit,
        page.offset
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map(|row| row.total).unwrap_or(0);
    let items = rows
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, String>>()
        .map_err(|error| sqlx::Error::Protocol(error.into()))?;
    Ok(Page { items, total })
}

pub async fn detail(pool: &PgPool, id: i64) -> Result<Option<LogDetail>, sqlx::Error> {
    let row = sqlx::query_file_as!(LogDetailRow, "sql/logs/detail.sql", id)
        .fetch_optional(pool)
        .await?;

    row.map(|row| {
        let log = RequestLogRow {
            id: row.id,
            occurred_at: row.occurred_at,
            rule_id: row.rule_id,
            target_id: row.target_id,
            action: row.action,
            method: row.method,
            path: row.path,
            query_string: row.query_string,
            content_type: row.content_type,
            body_format: row.body_format,
            request_headers: row.request_headers,
            response_headers: row.response_headers,
            http_status_code: row.http_status_code,
            latency_ms: row.latency_ms,
            error_message: row.error_message,
        };
        let snapshots = serde_json::from_value::<Vec<MessageSnapshot>>(row.snapshots)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        Ok(LogDetail {
            log: log.try_into().map_err(sqlx::Error::Protocol)?,
            snapshots,
        })
    })
    .transpose()
}

pub async fn insert(
    pool: &PgPool,
    request: &crate::domain::MockRequest,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    action: Option<RuleAction>,
    response: Option<&crate::domain::MockResponse>,
    latency_ms: Option<i64>,
    error_message: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let (status_code, response_headers) = response
        .map(|value| {
            (
                Some(value.status_code as i32),
                crate::domain::headers_to_json(&value.headers),
            )
        })
        .unwrap_or((None, serde_json::json!({})));

    Ok(sqlx::query_file!(
        "sql/logs/insert.sql",
        rule_id,
        target_id,
        action.map(RuleAction::as_str),
        &request.method,
        &request.path,
        (!request.query_string.is_empty()).then_some(&request.query_string),
        request.content_type.as_deref(),
        request.body_format.as_str(),
        &crate::domain::headers_to_json(&request.headers),
        &response_headers,
        status_code,
        latency_ms,
        error_message
    )
    .fetch_one(pool)
    .await?
    .id)
}

pub async fn insert_snapshot(
    pool: &PgPool,
    log_id: i64,
    kind: SnapshotKind,
    raw_body: &str,
    normalized_json: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "sql/logs/insert_snapshot.sql",
        log_id,
        kind.as_str(),
        raw_body,
        normalized_json
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_error(pool: &PgPool, id: i64, error_message: &str) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/logs/update_error.sql", id, error_message)
        .execute(pool)
        .await?;
    Ok(())
}

fn parse_action(value: &str) -> Result<RuleAction, String> {
    match value {
        "proxy" => Ok(RuleAction::Proxy),
        "static" => Ok(RuleAction::Static),
        other => Err(format!("unsupported log action: {other}")),
    }
}

fn parse_body_format(value: &str) -> Result<BodyFormat, String> {
    match value {
        "json" => Ok(BodyFormat::Json),
        "xml" => Ok(BodyFormat::Xml),
        "text" => Ok(BodyFormat::Text),
        other => Err(format!("unsupported body format: {other}")),
    }
}
