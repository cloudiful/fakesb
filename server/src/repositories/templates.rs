use sqlx::{FromRow, PgPool};

use crate::domain::{Page, PaginationParams, ResponseTemplate};

#[derive(Debug, FromRow)]
struct TemplateRow {
    id: i64,
    name: String,
    content_type: String,
    raw_template: String,
    format: String,
    status_code: i32,
    headers: serde_json::Value,
    enabled: bool,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TemplateRow> for ResponseTemplate {
    fn from(row: TemplateRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            content_type: row.content_type,
            raw_template: row.raw_template,
            format: row.format,
            status_code: row.status_code as u16,
            headers: row.headers,
            enabled: row.enabled,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct TemplateListRow {
    total: i64,
    id: i64,
    name: String,
    content_type: String,
    raw_template: String,
    format: String,
    status_code: i32,
    headers: serde_json::Value,
    enabled: bool,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TemplateListRow> for ResponseTemplate {
    fn from(row: TemplateListRow) -> Self {
        TemplateRow {
            id: row.id,
            name: row.name,
            content_type: row.content_type,
            raw_template: row.raw_template,
            format: row.format,
            status_code: row.status_code,
            headers: row.headers,
            enabled: row.enabled,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
        .into()
    }
}

pub async fn list(
    pool: &PgPool,
    page: PaginationParams,
) -> Result<Page<ResponseTemplate>, sqlx::Error> {
    let rows = sqlx::query_file_as!(
        TemplateListRow,
        "sql/templates/list.sql",
        page.limit,
        page.offset
    )
    .fetch_all(pool)
    .await?;
    let total = rows.first().map(|row| row.total).unwrap_or(0);
    Ok(Page {
        items: rows.into_iter().map(Into::into).collect(),
        total,
    })
}

pub async fn find_enabled(pool: &PgPool, id: i64) -> Result<Option<ResponseTemplate>, sqlx::Error> {
    sqlx::query_file_as!(TemplateRow, "sql/templates/find_enabled.sql", id)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Into::into))
}

pub async fn insert(
    pool: &PgPool,
    name: &str,
    content_type: &str,
    raw_template: &str,
    format: &str,
    status_code: u16,
    headers: &serde_json::Value,
    enabled: bool,
    note: Option<&str>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/templates/insert.sql",
        name,
        content_type,
        raw_template,
        format,
        status_code as i32,
        headers,
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
    name: &str,
    content_type: &str,
    raw_template: &str,
    format: &str,
    status_code: u16,
    headers: &serde_json::Value,
    enabled: bool,
    note: Option<&str>,
) -> Result<Option<i64>, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/templates/update.sql",
        id,
        name,
        content_type,
        raw_template,
        format,
        status_code as i32,
        headers,
        enabled,
        note
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.id))
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<ResponseTemplate>, sqlx::Error> {
    sqlx::query_file_as!(TemplateRow, "sql/templates/list_all.sql")
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
}

pub async fn upsert(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    name: &str,
    content_type: &str,
    raw_template: &str,
    format: &str,
    status_code: u16,
    headers: &serde_json::Value,
    enabled: bool,
    note: Option<&str>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/templates/upsert.sql",
        name,
        content_type,
        raw_template,
        format,
        status_code as i32,
        headers,
        enabled,
        note
    )
    .fetch_one(executor)
    .await?
    .id)
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!("sql/templates/delete.sql", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}
