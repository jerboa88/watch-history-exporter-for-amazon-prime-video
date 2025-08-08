use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceType {
    Simkl,
    Tmdb,
    Tvdb,
    Imdb,
    Mal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Tv,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataResult {
    pub ids: MediaIds,
    pub title: String,
    pub year: Option<String>,
    pub media_type: MediaType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaIds {
    pub simkl: Option<String>,
    pub tvdb: Option<String>,
    pub tmdb: Option<String>,
    pub imdb: Option<String>,
    pub mal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub simkl: RateLimit,
    pub tmdb: RateLimit,
    pub tvdb: RateLimit,
    pub imdb: RateLimit,
    pub mal: RateLimit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimit {
    pub calls: u32,
    pub per_seconds: u64,
}

pub type PriorityOrder = Vec<ServiceType>;