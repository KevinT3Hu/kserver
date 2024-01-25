#![allow(clippy::module_name_repetitions)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AnimeWatchListRequest {
    pub anime_id: i32,
    pub watch_list_name: String,
}

#[derive(Deserialize, Debug)]
pub struct WatchListRequest {
    pub watch_list_name: String,
}

#[derive(Deserialize, Debug)]
pub struct LogInRequest {
    pub otp: String,
}

#[derive(Deserialize, Debug)]
pub struct LogOutRequest {
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateEpisodeWatchedStateRequest {
    pub anime_id: i32,
    pub ep: i32,
    pub watched: bool,
}

#[derive(Deserialize, Debug)]
pub struct UpdateWatchListArchivedRequest {
    pub watch_list_name: String,
    pub archived: bool,
}

#[derive(Deserialize, Debug)]
pub struct UpdateAnimeVisibilityRequest {
    pub anime_id: i32,
    pub visible: bool,
}

#[derive(Deserialize, Debug)]
pub struct AnimeIdRequest {
    pub anime_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct GetAnimeStatesRequest {
    pub anime_ids: Vec<i32>,
}

#[derive(Deserialize, Debug)]
pub struct PostUpdateAnimeRatingRequest {
    pub anime_id: i32,
    pub rating: i32,
}
