use thiserror::Error;

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
