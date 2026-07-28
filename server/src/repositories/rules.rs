use sqlx::{FromRow, PgPool};

use crate::domain::{Page, PaginationParams, Rule, RuleMode};

#[derive(Debug, FromRow)]
pub struct RuleRow {
    pub id: i64,
    pub service_code: String,
    pub message_type: String,
    pub message_code: String,
    pub target_id: Option<i64>,
    pub mode: String,
    pub response_template_id: Option<i64>,
    pub priority: i32,
    pub enabled: bool,
    pub note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<RuleRow> for Rule {
    type Error = String;

    fn try_from(row: RuleRow) -> Result<Self, Self::Error> {
        let mode = match row.mode.as_str() {
            "passthrough" => RuleMode::Passthrough,
            "mock" => RuleMode::Mock,
            value => return Err(format!("unsupported rule mode: {value}")),
        };

        Ok(Self {
            id: row.id,
            service_code: row.service_code,
            message_type: row.message_type,
            message_code: row.message_code,
            target_id: row.target_id,
            mode,
            response_template_id: row.response_template_id,
            priority: row.priority,
            enabled: row.enabled,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, FromRow)]
struct RuleListRow {
    total: i64,
    id: i64,
    service_code: String,
    message_type: String,
    message_code: String,
    target_id: Option<i64>,
    mode: String,
    response_template_id: Option<i64>,
    priority: i32,
    enabled: bool,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<RuleListRow> for Rule {
    type Error = String;

    fn try_from(row: RuleListRow) -> Result<Self, Self::Error> {
        RuleRow {
            id: row.id,
            service_code: row.service_code,
            message_type: row.message_type,
            message_code: row.message_code,
            target_id: row.target_id,
            mode: row.mode,
            response_template_id: row.response_template_id,
            priority: row.priority,
            enabled: row.enabled,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
        .try_into()
    }
}

pub async fn list(pool: &PgPool, page: PaginationParams) -> Result<Page<Rule>, sqlx::Error> {
    let rows = sqlx::query_file_as!(RuleListRow, "sql/rules/list.sql", page.limit, page.offset)
        .fetch_all(pool)
        .await?;

    let total = rows.first().map(|row| row.total).unwrap_or(0);
    let items = rows
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, String>>()
        .map_err(|err| sqlx::Error::Protocol(err.into()))?;

    Ok(Page { items, total })
}

pub async fn find_match(
    pool: &PgPool,
    service_code: &str,
    message_type: &str,
    message_code: &str,
) -> Result<Option<Rule>, sqlx::Error> {
    let row = sqlx::query_file_as!(
        RuleRow,
        "sql/rules/find_match.sql",
        service_code,
        message_type,
        message_code
    )
    .fetch_optional(pool)
    .await?;

    row.map(TryInto::try_into)
        .transpose()
        .map_err(|err: String| sqlx::Error::Protocol(err.into()))
}

pub async fn insert(
    pool: &PgPool,
    service_code: &str,
    message_type: &str,
    message_code: &str,
    target_id: Option<i64>,
    mode: &str,
    response_template_id: Option<i64>,
    priority: i32,
    enabled: bool,
    note: Option<&str>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/rules/insert.sql",
        service_code,
        message_type,
        message_code,
        target_id,
        mode,
        response_template_id,
        priority,
        enabled,
        note
    )
    .fetch_one(pool)
    .await?
    .id)
}

pub async fn update(
    pool: &PgPool,
    id: i64,
    service_code: &str,
    message_type: &str,
    message_code: &str,
    target_id: Option<i64>,
    mode: &str,
    response_template_id: Option<i64>,
    priority: i32,
    enabled: bool,
    note: Option<&str>,
) -> Result<Option<i64>, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/rules/update.sql",
        id,
        service_code,
        message_type,
        message_code,
        target_id,
        mode,
        response_template_id,
        priority,
        enabled,
        note
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.id))
}
