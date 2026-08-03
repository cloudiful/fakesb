use sqlx::{FromRow, PgPool};

use crate::domain::SequenceStep;

#[derive(Debug, FromRow)]
pub struct SequenceStepRow {
    pub id: i64,
    pub rule_id: i64,
    pub step_index: i32,
    pub template_id: i64,
    pub template_name: Option<String>,
}

impl From<SequenceStepRow> for SequenceStep {
    fn from(row: SequenceStepRow) -> Self {
        Self {
            id: row.id,
            rule_id: row.rule_id,
            step_index: row.step_index,
            template_id: row.template_id,
            template_name: row.template_name,
        }
    }
}

pub async fn list_by_rule_ids(
    pool: &PgPool,
    rule_ids: &[i64],
) -> Result<Vec<SequenceStep>, sqlx::Error> {
    if rule_ids.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_file_as!(
        SequenceStepRow,
        "sql/sequences/list_by_rule_ids.sql",
        rule_ids
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(Into::into).collect())
}

pub async fn attach(pool: &PgPool, rules: &mut [crate::domain::Rule]) {
    let ids: Vec<i64> = rules.iter().map(|rule| rule.id).collect();
    let steps = match list_by_rule_ids(pool, &ids).await {
        Ok(steps) => steps,
        Err(_) => return,
    };
    for rule in rules {
        rule.sequence_steps = steps
            .iter()
            .filter(|step| step.rule_id == rule.id)
            .cloned()
            .collect();
    }
}

pub async fn replace(
    executor: &mut sqlx::PgConnection,
    rule_id: i64,
    steps: &[(i32, i64)],
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/sequences/delete_by_rule.sql", rule_id)
        .execute(&mut *executor)
        .await?;
    for (index, template_id) in steps {
        sqlx::query_file!("sql/sequences/insert_step.sql", rule_id, index, template_id)
            .execute(&mut *executor)
            .await?;
    }
    Ok(())
}

pub async fn count_by_template(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    template_id: i64,
) -> Result<i64, sqlx::Error> {
    Ok(
        sqlx::query_file!("sql/sequences/count_by_template.sql", template_id)
            .fetch_one(executor)
            .await?
            .count,
    )
}

pub async fn next_count(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    rule_id: i64,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_file!("sql/sequences/next_count.sql", rule_id)
        .fetch_one(executor)
        .await?
        .sequence_count)
}

pub async fn current_count(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    rule_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    Ok(
        sqlx::query_file!("sql/sequences/current_count.sql", rule_id)
            .fetch_optional(executor)
            .await?
            .map(|row| row.sequence_count),
    )
}

pub async fn reset_count(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    rule_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("sql/sequences/reset_count.sql", rule_id)
        .execute(executor)
        .await?;
    Ok(())
}
