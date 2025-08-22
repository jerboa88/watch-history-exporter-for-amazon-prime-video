use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchHistoryItem {
    pub simkl_id: Option<String>,
    pub tvdb_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub mal_id: Option<String>,
    pub media_type: MediaType,
    pub title: String,
    pub year: Option<String>,
    pub episode: Option<String>,
    pub watch_status: WatchStatus,
    pub date: String,
    pub rating: Option<u8>,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    Tv,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchStatus {
    Completed,
    Watching,
    Planned,
    Dropped,
}