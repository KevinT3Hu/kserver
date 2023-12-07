use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{get, post},
    Json, Router,
};
use tracing::{error, event};

use crate::{
    auth_middleware,
    helper::db_error::DbError,
    internal_error,
    model::{
        request::{
            AnimeWatchListRequest, WatchListRequest,
            UpdateAnimeVisibilityRequest, UpdateEpisodeWatchedStateRequest,
            UpdateWatchListArchivedRequest, GetAnimeStatesRequest,
        },
        AnimeState, WatchList, AnimeItem,
    },
    status, AppState,
};

use super::Result;

pub const PATH: &str = "/anime";

pub fn create_router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/insert_anime_item", post(post_insert_item))
        .route("/add_item_to_watch_list", post(post_add_item_to_watch_list))
        .route("/add_new_watch_list", post(post_add_new_watch_list))
        .route(
            "/update_episode_watched_state",
            post(post_update_episode_watched_state),
        )
        .route(
            "/update_anime_visibility",
            post(post_update_anime_visibility),
        )
        .route(
            "/update_watch_list_archived",
            post(post_update_watch_list_archived),
        )
        .route("/delete_watch_list", post(post_delete_watch_list))
        .route("/delete_anime_state_from_watch_list", post(post_delete_anime_state_from_watch_list))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .route("/list", get(get_all_list))
        .route("/get/:anime_id", get(get_query_anime_by_id))
        .route("/get_anime_states", post(post_query_anime_states))
        .route("/all", get(get_query_all_anime_states))
        .route("/get_watch_list/:watch_list_name", get(get_query_watch_list_by_name))
}

async fn get_all_list(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<WatchList>>> {
    let db = app_state.db_helper.clone();

    let result = db.get_all_list().await;
    if let Err(e) = &result {
        internal_error!("Error: {:?}", e);
    }

    Ok(Json(result.unwrap()))
}

async fn post_insert_item(
    State(app_state): State<AppState>,
    Json(req): Json<AnimeItem>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();
    event!(tracing::Level::INFO, "Inserting anime item: {:?}", req);

    let result = db.insert_anime_item(req).await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(anime_id) => {
                return Err(status!(NOT_FOUND, "AnimeNotFound:{}", anime_id))
            }
            DbError::EpisodeNotFound(_) => return Err(status!(INTERNAL_SERVER_ERROR)),
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(StatusCode::CREATED)
}

async fn post_add_item_to_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<AnimeWatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();
    event!(tracing::Level::INFO, "Adding anime item to watch list: {:?}", req);

    let result = db
        .add_item_to_watch_list(req.anime_id, &req.watch_list_name)
        .await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(anime_id) => {
                return Err(status!(BAD_REQUEST, "AnimeNotFound:{}", anime_id))
            }
            DbError::EpisodeNotFound(_) => return Err(status!(INTERNAL_SERVER_ERROR)),
            DbError::WatchListNotFound(_) => {
                internal_error!();
            },
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(StatusCode::CREATED)
}

async fn post_add_new_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<WatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    let result = db.add_new_watch_list(&req.watch_list_name).await;
    if let Err(e) = &result {
        internal_error!("Error: {:?}", e);
    }

    Ok(StatusCode::CREATED)
}

async fn post_update_episode_watched_state(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateEpisodeWatchedStateRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    let result = db
        .update_episode_watched_state(req.anime_id,req.ep, req.watched)
        .await;
    if let Err(e) = &result {
        error!("Error: {:?}", e);
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(episode_id) => {
                return Err(status!(BAD_REQUEST, "EpisodeNotFound:{}", episode_id))
            }
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(StatusCode::OK)
}

async fn post_update_watch_list_archived(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateWatchListArchivedRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    let result = db
        .update_watch_list_archive_state(&req.watch_list_name, req.archived)
        .await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(_) => {
                internal_error!();
            }
            DbError::WatchListNotFound(watch_list_name) => {
                return Err(status!(BAD_REQUEST, "WatchListNotFound:{}", watch_list_name))
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(StatusCode::OK)
}

async fn post_update_anime_visibility(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateAnimeVisibilityRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    let result = db.update_anime_visibility(req.anime_id, req.visible).await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(_) => {
                internal_error!();
            }
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(StatusCode::OK)
}

async fn get_query_anime_by_id(
    State(app_state): State<AppState>,
    Path(anime_id): Path<i32>,
) -> Result<Json<AnimeState>> {
    let db = app_state.db_helper.clone();

    let result = db.query_anime_by_id(anime_id).await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(anime_id) => {
                return Err(status!(NOT_FOUND, "AnimeNotFound:{}", anime_id))
            }
            DbError::EpisodeNotFound(_) => return Err(status!(INTERNAL_SERVER_ERROR)),
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(Json(result.unwrap()))
}

async fn post_delete_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<WatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    let result = db.delete_watch_list(&req.watch_list_name).await;
    if let Err(e) = &result {
        internal_error!("Error: {:?}", e);
    }

    Ok(StatusCode::OK)
}

async fn post_query_anime_states(
    State(app_state): State<AppState>,
    Json(req): Json<GetAnimeStatesRequest>,
) -> Result<Json<Vec<AnimeState>>> {
    let db = app_state.db_helper.clone();

    let result = db.query_anime_states_by_ids(&req.anime_ids).await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(_) => {
                internal_error!();
            }
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(Json(result.unwrap()))
}

async fn post_delete_anime_state_from_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<AnimeWatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    let result = db
        .delete_anime_state_from_watch_list(req.anime_id, &req.watch_list_name)
        .await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(_) => {
                internal_error!();
            }
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(StatusCode::OK)
}

async fn get_query_all_anime_states(
    State(app_state): State<AppState>
) -> Result<Json<Vec<AnimeState>>>{
    let db = app_state.db_helper.clone();

    let result = db.query_all_animes().await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(_) => {
                internal_error!();
            }
            DbError::WatchListNotFound(_) => {
                internal_error!();
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(Json(result.unwrap()))
}

async fn get_query_watch_list_by_name(
    State(app_state): State<AppState>,
    Path(watch_list_name): Path<String>,
) -> Result<Json<WatchList>> {
    let db = app_state.db_helper.clone();

    let result = db.get_watch_list(&watch_list_name).await;
    if let Err(e) = &result {
        match e {
            DbError::AnimeNotFound(_) => {
                internal_error!();
            }
            DbError::EpisodeNotFound(_) => {
                internal_error!();
            }
            DbError::WatchListNotFound(watch_list_name) => {
                return Err(status!(NOT_FOUND, "WatchListNotFound:{}", watch_list_name))
            }
            DbError::PostgresError(e) => {
                internal_error!("Error: {:?}", e);
            }
        }
    }

    Ok(Json(result.unwrap()))
}