use sqlx::{FromRow, PgPool};

use crate::domain::{Page, PaginationParams, Rule, RuleAction, RuleMatcher};

#[derive(Debug, FromRow)]
struct RuleRow {
    id: i64,
    matcher: serde_json::Value,
    target_id: Option<i64>,
    action: String,
    response_template_id: Option<i64>,
    priority: i32,
    enabled: bool,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<RuleRow> for Rule {
    type Error = String;

    fn try_from(row: RuleRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            matcher: serde_json::from_value::<RuleMatcher>(row.matcher)
                .map_err(|error| format!("invalid stored matcher: {error}"))?,
            target_id: row.target_id,
            action: parse_action(&row.action)?,
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
    matcher: serde_json::Value,
    target_id: Option<i64>,
    action: String,
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
            matcher: row.matcher,
            target_id: row.target_id,
            action: row.action,
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
        .map_err(|error| sqlx::Error::Protocol(error.into()))?;
    Ok(Page { items, total })
}

pub async fn list_enabled(pool: &PgPool) -> Result<Vec<Rule>, sqlx::Error> {
    let rows = sqlx::query_file_as!(RuleRow, "sql/rules/enabled.sql")
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, String>>()
        .map_err(|error| sqlx::Error::Protocol(error.into()))
}

pub async fn insert(
    pool: &PgPool,
    matcher: &RuleMatcher,
    target_id: Option<i64>,
    action: RuleAction,
    response_template_id: Option<i64>,
    priority: i32,
    enabled: bool,
    note: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let matcher = serde_json::to_value(matcher)
        .map_err(|error| sqlx::Error::Protocol(error.to_string().into()))?;
    Ok(sqlx::query_file!(
        "sql/rules/insert.sql",
        matcher,
        target_id,
        action.as_str(),
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
    matcher: &RuleMatcher,
    target_id: Option<i64>,
    action: RuleAction,
    response_template_id: Option<i64>,
    priority: i32,
    enabled: bool,
    note: Option<&str>,
) -> Result<Option<i64>, sqlx::Error> {
    let matcher = serde_json::to_value(matcher)
        .map_err(|error| sqlx::Error::Protocol(error.to_string().into()))?;
    Ok(sqlx::query_file!(
        "sql/rules/update.sql",
        id,
        matcher,
        target_id,
        action.as_str(),
        response_template_id,
        priority,
        enabled,
        note
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.id))
}

fn parse_action(value: &str) -> Result<RuleAction, String> {
    match value {
        "proxy" => Ok(RuleAction::Proxy),
        "static" => Ok(RuleAction::Static),
        other => Err(format!("unsupported rule action: {other}")),
    }
}
