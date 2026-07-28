use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::domain::{
    LogDetail, MessageSnapshot, Page, PaginationParams, RequestLog, RuleMode, SnapshotKind,
};

#[derive(Debug, FromRow)]
struct RequestLogRow {
    id: i64,
    occurred_at: DateTime<Utc>,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    mode: Option<String>,
    service_code: String,
    message_type: String,
    message_code: String,
    http_status_code: Option<String>,
    ret_code: Option<String>,
    ret_msg: Option<String>,
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
            mode: parse_mode(row.mode)?,
            service_code: row.service_code,
            message_type: row.message_type,
            message_code: row.message_code,
            http_status_code: row.http_status_code,
            ret_code: row.ret_code,
            ret_msg: row.ret_msg,
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
    mode: Option<String>,
    service_code: String,
    message_type: String,
    message_code: String,
    http_status_code: Option<String>,
    ret_code: Option<String>,
    ret_msg: Option<String>,
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
            mode: row.mode,
            service_code: row.service_code,
            message_type: row.message_type,
            message_code: row.message_code,
            http_status_code: row.http_status_code,
            ret_code: row.ret_code,
            ret_msg: row.ret_msg,
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
    mode: Option<String>,
    service_code: String,
    message_type: String,
    message_code: String,
    http_status_code: Option<String>,
    ret_code: Option<String>,
    ret_msg: Option<String>,
    latency_ms: Option<i64>,
    error_message: Option<String>,
    snapshots: serde_json::Value,
}

pub async fn list(
    pool: &PgPool,
    page: PaginationParams,
    service_code: Option<&str>,
    message_type: Option<&str>,
    message_code: Option<&str>,
    mode: Option<&str>,
    ret_code: Option<&str>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
) -> Result<Page<RequestLog>, sqlx::Error> {
    let rows = sqlx::query_file_as!(
        RequestLogListRow,
        "sql/logs/list.sql",
        service_code,
        message_type,
        message_code,
        mode,
        ret_code,
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
        .map_err(sqlx::Error::Protocol)?;

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
            mode: row.mode,
            service_code: row.service_code,
            message_type: row.message_type,
            message_code: row.message_code,
            http_status_code: row.http_status_code,
            ret_code: row.ret_code,
            ret_msg: row.ret_msg,
            latency_ms: row.latency_ms,
            error_message: row.error_message,
        };
        let snapshots = serde_json::from_value::<Vec<MessageSnapshot>>(row.snapshots)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        Ok(LogDetail {
            log: log.try_into().map_err(sqlx::Error::Protocol)?,
            snapshots,
        })
    })
    .transpose()
}

pub async fn insert(
    pool: &PgPool,
    request: &crate::domain::ParsedEsbMessage,
    rule_id: Option<i64>,
    target_id: Option<i64>,
    mode: Option<&RuleMode>,
    response: Option<&crate::domain::EsbResponse>,
    latency_ms: Option<i64>,
    error_message: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let (status_code, ret_code, ret_msg) = response
        .map(|value| {
            (
                Some(value.status_code.to_string()),
                value.ret_code.as_deref(),
                value.ret_msg.as_deref(),
            )
        })
        .unwrap_or((None, None, None));

    Ok(sqlx::query_file!(
        "sql/logs/insert.sql",
        rule_id,
        target_id,
        mode.map(RuleMode::as_str),
        &request.service_code,
        &request.message_type,
        &request.message_code,
        status_code,
        ret_code,
        ret_msg,
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

fn parse_mode(mode: Option<String>) -> Result<Option<RuleMode>, String> {
    mode.map(|value| match value.as_str() {
        "passthrough" => Ok(RuleMode::Passthrough),
        "mock" => Ok(RuleMode::Mock),
        other => Err(format!("unsupported log mode: {other}")),
    })
    .transpose()
}
