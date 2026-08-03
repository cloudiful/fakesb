use sqlx::{FromRow, PgPool};

use crate::domain::{Page, PaginationParams, Target};

#[derive(Debug, FromRow)]
struct TargetRow {
    id: i64,
    name: String,
    base_url: String,
    enabled: bool,
    timeout_ms: i32,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TargetRow> for Target {
    fn from(row: TargetRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            base_url: row.base_url,
            enabled: row.enabled,
            timeout_ms: row.timeout_ms,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct TargetListRow {
    total: i64,
    id: i64,
    name: String,
    base_url: String,
    enabled: bool,
    timeout_ms: i32,
    note: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<TargetListRow> for Target {
    fn from(row: TargetListRow) -> Self {
        TargetRow {
            id: row.id,
            name: row.name,
            base_url: row.base_url,
            enabled: row.enabled,
            timeout_ms: row.timeout_ms,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
        .into()
    }
}

pub async fn list(pool: &PgPool, page: PaginationParams) -> Result<Page<Target>, sqlx::Error> {
    let rows = sqlx::query_file_as!(
        TargetListRow,
        "sql/targets/list.sql",
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

pub async fn find_enabled(pool: &PgPool, id: i64) -> Result<Option<Target>, sqlx::Error> {
    sqlx::query_file_as!(TargetRow, "sql/targets/find_enabled.sql", id)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Into::into))
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<Target>, sqlx::Error> {
    sqlx::query_file_as!(TargetRow, "sql/targets/list_all.sql")
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
}

pub async fn upsert(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    name: &str,
    base_url: &str,
    enabled: bool,
    timeout_ms: i32,
    note: Option<&str>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/targets/upsert.sql",
        name,
        base_url,
        enabled,
        timeout_ms,
        note
    )
    .fetch_one(executor)
    .await?
    .id)
}

pub async fn insert(
    pool: &PgPool,
    name: &str,
    base_url: &str,
    enabled: bool,
    timeout_ms: i32,
    note: Option<&str>,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/targets/insert.sql",
        name,
        base_url,
        enabled,
        timeout_ms,
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
    base_url: &str,
    enabled: bool,
    timeout_ms: i32,
    note: Option<&str>,
) -> Result<Option<i64>, sqlx::Error> {
    Ok(sqlx::query_file!(
        "sql/targets/update.sql",
        id,
        name,
        base_url,
        enabled,
        timeout_ms,
        note
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.id))
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!("sql/targets/delete.sql", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}
