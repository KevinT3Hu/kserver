use axum::http::StatusCode;
use thiserror::Error;

use crate::{router::ComplexResponse, status};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Cannot find anime with id {0}")]
    AnimeNotFound(i32),

    #[error("Cannot find episode with id {0}")]
    EpisodeNotFound(i32),

    #[error("Database error {0}")]
    PostgresError(#[from] tokio_postgres::Error),

    #[error("Cannot find watch list with id {0}")]
    WatchListNotFound(String),
}

impl From<DbError> for ComplexResponse {
    fn from(value: DbError) -> Self {
        tracing::error!("Error: {:?}", value);
        match value {
            DbError::AnimeNotFound(id) => status!(NOT_FOUND, "Cannot find anime with id {}", id),
            DbError::EpisodeNotFound(id) => {
                status!(NOT_FOUND, "Cannot find episode with id {}", id)
            }
            DbError::PostgresError(e) => status!(INTERNAL_SERVER_ERROR, "Database error: {:?}", e),
            DbError::WatchListNotFound(id) => {
                status!(NOT_FOUND, "Cannot find watch list with id {}", id)
            }
        }
    }
}
