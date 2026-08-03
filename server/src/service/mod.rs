mod admin;
mod dispatch;
mod logging;
mod mock;
mod parser;
mod passthrough;
mod response;

#[path = "../matching.rs"]
pub(crate) mod matching;

pub(crate) use admin::{LogQuery, RuleInput, SequenceStepInput};
pub(crate) use dispatch::dispatch;
pub(crate) use parser::{from_parts, parse_request, query_string};

use reqwest::Client;
use sqlx::PgPool;
use thiserror::Error;

use crate::migrations::MIGRATOR;

#[derive(Clone)]
pub struct AppServices {
    pool: PgPool,
    client: Client,
    default_target_id: Option<i64>,
    request_timeout_ms: u64,
}

impl AppServices {
    pub fn new(pool: PgPool, default_target_id: Option<i64>, request_timeout_ms: u64) -> Self {
        Self {
            pool,
            client: Client::new(),
            default_target_id,
            request_timeout_ms,
        }
    }

    pub async fn initialize(&self) -> Result<(), ServiceError> {
        MIGRATOR.run(&self.pool).await.map_err(ServiceError::from)
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn default_target_id(&self) -> Option<i64> {
        self.default_target_id
    }

    pub(crate) fn request_timeout_ms(&self) -> u64 {
        self.request_timeout_ms
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("template error: {0}")]
    Template(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("remote error: {0}")]
    Remote(String),
    #[error("request parse error: {0}")]
    Parse(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("no matching rule")]
    NoMatch,
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for ServiceError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::Database(err.to_string())
    }
}
