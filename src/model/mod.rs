use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    hash::Hash,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Row;
pub mod request;

#[derive(Serialize, Deserialize)]
pub struct WatchList {
    pub title: String,
    pub archived: bool,
    pub animes: Vec<i32>, // Corresponding to anime id
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rating {
    pub rank: i32,
    pub total: i32,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageSet {
    pub large: String,
    pub common: String,
    pub medium: String,
    pub small: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeItem {
    pub id: i32,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub date: String,
    pub eps: i32,
    pub total_episodes: i32,
    pub images: ImageSet,
    #[serde(skip_serializing, default)]
    pub tags: Option<Vec<Tag>>,
    #[serde(skip_serializing, default)]
    pub rating: Option<Rating>,
}

// For use in HashSet
pub enum Float {
    Int(i32),
    Half(i32),
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Half(l0), Self::Half(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for Float {}

impl Hash for Float {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(i) => (i * 10).hash(state),
            Self::Half(i) => (i * 10 + 5).hash(state),
        }
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => {
                write!(f, "{}", i)
            }
            Self::Half(i) => {
                write!(f, "{}.5", i)
            }
        }
    }
}

impl Float {
    pub fn new(i: f32) -> Self {
        let diff = i - i.floor();
        if diff < 0.25 {
            Self::Int(i as i32)
        } else if diff > 0.75 {
            Self::Int(i as i32 + 1)
        } else {
            Self::Half(i as i32)
        }
    }
}

impl Serialize for Float {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Int(i) => serializer.serialize_i32(*i),
            Self::Half(i) => serializer.serialize_f32((*i as f32 + 0.5) as f32),
        }
    }
}

impl<'de> Deserialize<'de> for Float {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let f = f32::deserialize(deserializer)?;
        Ok(Self::new(f))
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnimeState {
    pub anime_id: i32,
    pub anime_item: AnimeItem,
    pub favorite: bool,
    pub watched_episodes: HashSet<Float>,
    pub visibility: bool,
    pub rating: Option<i32>,
}

impl WatchList {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            archived: false,
            animes: Vec::new(),
        }
    }
}

impl From<&Row> for WatchList {
    fn from(value: &Row) -> Self {
        Self {
            title: value.get(0),
            archived: value.get(1),
            animes: value.get(2),
        }
    }
}

impl From<&Row> for AnimeState {
    fn from(value: &Row) -> Self {
        let watched_episodes: Value = value.get(3);
        let watched_episodes: HashSet<Float> = serde_json::from_value(watched_episodes).unwrap();

        Self {
            anime_id: value.get(0),
            anime_item: serde_json::from_value(value.get(1)).unwrap(),
            favorite: value.get(2),
            watched_episodes,
            visibility: value.get(4),
            rating: value.get(5),
        }
    }
}
