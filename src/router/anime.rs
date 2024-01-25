use axum::{
    extract::{State, Query},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{get, post},
    Json, Router,
};
use tracing::event;

use crate::{
    auth_middleware,
    model::{
        request::{
            AnimeWatchListRequest, GetAnimeStatesRequest, PostUpdateAnimeRatingRequest,
            UpdateAnimeVisibilityRequest, UpdateEpisodeWatchedStateRequest,
            UpdateWatchListArchivedRequest, WatchListRequest, AnimeIdRequest,
        },
        AnimeItem, AnimeState, WatchList,
    },
    AppState,
};

use super::Result;

pub const PATH: &str = "/anime";

pub fn create(state: &AppState) -> Router<AppState> {
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
        .route(
            "/delete_anime_state_from_watch_list",
            post(post_delete_anime_state_from_watch_list),
        )
        .route("/update_anime_rating", post(post_update_anime_rating))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .route("/list", get(get_all_list))
        .route("/get", get(get_query_anime_by_id))
        .route("/get_anime_states", post(post_query_anime_states))
        .route("/all", get(get_query_all_anime_states))
        .route(
            "/get_watch_list",
            get(get_query_watch_list_by_name),
        )
}

async fn get_all_list(State(app_state): State<AppState>) -> Result<Json<Vec<WatchList>>> {
    let db = app_state.db_helper.clone();

    let result = db.get_all_list().await?;

    Ok(Json(result))
}

async fn post_insert_item(
    State(app_state): State<AppState>,
    Json(req): Json<AnimeItem>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();
    event!(tracing::Level::INFO, "Inserting anime item: {:?}", req);

    db.insert_anime_item(req).await?;

    Ok(StatusCode::CREATED)
}

async fn post_add_item_to_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<AnimeWatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();
    event!(
        tracing::Level::INFO,
        "Adding anime item to watch list: {:?}",
        req
    );

    db.add_item_to_watch_list(req.anime_id, &req.watch_list_name)
        .await?;

    Ok(StatusCode::CREATED)
}

async fn post_add_new_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<WatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    db.add_new_watch_list(&req.watch_list_name).await?;

    Ok(StatusCode::CREATED)
}

async fn post_update_episode_watched_state(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateEpisodeWatchedStateRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    db.update_episode_watched_state(req.anime_id, req.ep, req.watched)
        .await?;

    Ok(StatusCode::OK)
}

async fn post_update_watch_list_archived(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateWatchListArchivedRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    db.update_watch_list_archive_state(&req.watch_list_name, req.archived)
        .await?;

    Ok(StatusCode::OK)
}

async fn post_update_anime_visibility(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateAnimeVisibilityRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    db.update_anime_visibility(req.anime_id, req.visible)
        .await?;

    Ok(StatusCode::OK)
}

async fn get_query_anime_by_id(
    State(app_state): State<AppState>,
    Query(AnimeIdRequest{anime_id}): Query<AnimeIdRequest>,
) -> Result<Json<AnimeState>> {
    let db = app_state.db_helper.clone();

    let result = db.query_anime_by_id(anime_id).await?;

    Ok(Json(result))
}

async fn post_delete_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<WatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    db.delete_watch_list(&req.watch_list_name).await?;

    Ok(StatusCode::OK)
}

async fn post_query_anime_states(
    State(app_state): State<AppState>,
    Json(req): Json<GetAnimeStatesRequest>,
) -> Result<Json<Vec<AnimeState>>> {
    let db = app_state.db_helper.clone();

    let result = db.query_anime_states_by_ids(&req.anime_ids).await?;

    Ok(Json(result))
}

async fn post_delete_anime_state_from_watch_list(
    State(app_state): State<AppState>,
    Json(req): Json<AnimeWatchListRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();

    db.delete_anime_state_from_watch_list(req.anime_id, &req.watch_list_name)
        .await?;

    Ok(StatusCode::OK)
}

async fn get_query_all_anime_states(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<AnimeState>>> {
    let db = app_state.db_helper.clone();

    let result = db.query_all_animes().await?;

    Ok(Json(result))
}

async fn get_query_watch_list_by_name(
    State(app_state): State<AppState>,
    Query(WatchListRequest{watch_list_name}): Query<WatchListRequest>,
) -> Result<Json<WatchList>> {
    let db = app_state.db_helper.clone();

    let result = db.get_watch_list(&watch_list_name).await?;

    Ok(Json(result))
}

async fn post_update_anime_rating(
    State(app_state): State<AppState>,
    Json(PostUpdateAnimeRatingRequest { anime_id, rating }): Json<PostUpdateAnimeRatingRequest>,
) -> Result<StatusCode> {
    let db = app_state.db_helper.clone();
    db.update_anime_rating(anime_id, rating).await?;
    Ok(StatusCode::OK)
}
