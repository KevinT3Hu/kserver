use serde_json::Value;
use tokio_postgres::NoTls;
use std::{sync::Arc, collections::HashSet};
use tracing::info;

use crate::model::{AnimeItem, AnimeState, WatchList};

use super::db_error::DbError;

#[derive(Clone)]
pub struct DbHelper {
    anime_db: Arc<tokio_postgres::Client>,
}

type Result<T> = std::result::Result<T, DbError>;

impl DbHelper {
    pub async fn new() -> Self {
        info!("Start creating database helper...");
        let (client,connection) = tokio_postgres::connect(&std::env::var("PG_URI").unwrap(), NoTls).await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("connection error: {}", e);
            }
        });
        info!("Database helper created");
        Self {
            anime_db: Arc::new(client),
        }
    }

    pub async fn get_all_list(&self) -> Result<Vec<WatchList>> {
        let client = self.anime_db.clone();
        let rows = client.query("SELECT * FROM anime_list", &[]).await?;
        let rows = rows.iter().map(|row| {
            row.into()
        }).collect();

        Ok(rows)
    }

    pub async fn query_anime_by_id(&self, anime_id: i32) -> Result<AnimeState> {
        let client = self.anime_db.clone();
        let rows = client.query("SELECT * FROM anime_state WHERE anime_id = $1", &[&anime_id]).await?;
        let ret = (&rows[0]).into();
        Ok(ret)
    }

    pub async fn insert_anime_item(&self, anime_item: AnimeItem) -> Result<()> {
        let client = self.anime_db.clone();
        let item_jsonb = serde_json::to_value(&anime_item).unwrap();
        client.execute("INSERT INTO anime_state (anime_id,anime_item) VALUES($1,$2)", &[&anime_item.id, &item_jsonb]).await?;
        Ok(())
    }

    pub async fn update_episode_watched_state(&self, anime_id: i32, ep:i32, watched: bool) -> Result<()> {
        let client = self.anime_db.clone();
        let watched_episode = client.query("SELECT watched_episodes FROM anime_state WHERE anime_id = $1", &[&anime_id]).await?;
        let watched_episode: Value = watched_episode[0].get(0);
        let mut watched_episode:HashSet<_> = serde_json::from_value(watched_episode).unwrap();
        if watched {
            watched_episode.insert(ep);
        } else {
            watched_episode.remove(&ep);
        }

        let watched_episode = serde_json::to_value(&watched_episode).unwrap();
        client.execute("UPDATE anime_state SET watched_episodes = $1 WHERE anime_id = $2", &[&watched_episode, &anime_id]).await?;

        Ok(())

    }

    pub async fn add_item_to_watch_list(&self, anime_id: i32, watch_list_name: &str) -> Result<()> {
        let client = self.anime_db.clone();
        let stmt = client.prepare("UPDATE anime_list SET animes = array_append(animes, $1) WHERE title = $2").await?;
        client.execute(&stmt, &[&anime_id, &watch_list_name]).await?;
        Ok(())
    }

    pub async fn add_new_watch_list(&self, watch_list_name: &str) -> Result<()> {
        let client = self.anime_db.clone();
        let animes:Vec<i32> = Vec::new();
        client.execute("INSERT INTO anime_list VALUES($1,$2,$3)", &[&watch_list_name, &false, &animes]).await?;
        Ok(())
    }

    pub async fn update_watch_list_archive_state(
        &self,
        watch_list_name: &str,
        archived: bool,
    ) -> Result<()> {
        let client = self.anime_db.clone();
        let stmt = client.prepare("UPDATE anime_list SET archived = $1 WHERE title = $2").await?;
        client.execute(&stmt, &[&archived, &watch_list_name]).await?;
        Ok(())
    }

    pub async fn update_anime_visibility(&self, anime_id: i32, visibility: bool) -> Result<()> {
        let client = self.anime_db.clone();
        let stmt = client.prepare("UPDATE anime_state SET visibility = $1 WHERE anime_id = $2").await?;
        client.execute(&stmt, &[&visibility, &anime_id]).await?;
        Ok(())
    }

    pub async fn delete_watch_list(&self, watch_list_name: &str) -> Result<()> {
        let client = self.anime_db.clone();
        client.execute("DELETE FROM anime_list WHERE title = $1", &[&watch_list_name]).await?;
        Ok(())
    }

    pub async fn query_anime_states_by_ids(&self, anime_ids:&Vec<i32>) -> Result<Vec<AnimeState>> {
        let client = self.anime_db.clone();
        let stmt = client.prepare("SELECT * FROM anime_state WHERE anime_id = ANY($1)").await?;
        let rows = client.query(&stmt, &[&anime_ids]).await?;
        let ret = rows.iter().map(|row| {
            row.into()
        }).collect();
        Ok(ret)
    }

    pub async fn delete_anime_state_from_watch_list(&self, anime_id: i32, watch_list_name: &str) -> Result<()> {
        let client = self.anime_db.clone();
        let stmt = client.prepare("UPDATE anime_list SET animes = array_remove(animes, $1) WHERE title = $2").await?;
        client.execute(&stmt, &[&anime_id, &watch_list_name]).await?;

        // if the anime is not in any watch list, delete it from anime_state
        let stmt = client.prepare("SELECT * FROM anime_list WHERE animes @> ARRAY[$1]").await?;
        let rows = client.query(&stmt, &[&anime_id]).await?;
        if rows.len() == 0 {
            client.execute("DELETE FROM anime_state WHERE anime_id = $1", &[&anime_id]).await?;
        }

        Ok(())
    }

    pub async fn query_all_animes(&self) -> Result<Vec<AnimeState>> {
        let client = self.anime_db.clone();
        let rows = client.query("SELECT * FROM anime_state", &[]).await?;
        let ret = rows.iter().map(|row| {
            row.into()
        }).collect();

        Ok(ret)
    }

    pub async fn get_watch_list(&self, watch_list_name: &str) -> Result<WatchList> {
        let client = self.anime_db.clone();
        let rows = client.query("SELECT * FROM anime_list WHERE title = $1", &[&watch_list_name]).await?;
        let ret = (&rows[0]).into();
        Ok(ret)
    }
}
