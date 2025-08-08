use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::{TmdbConfig, RateLimit},
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider},
};

pub struct TmdbClient {
    client: Client,
    config: TmdbConfig,
    rate_limit: RateLimit,
}

impl TmdbClient {
    pub fn new(config: TmdbConfig, rate_limit: RateLimit) -> Self {
        Self {
            client: Client::new(),
            config,
            rate_limit,
        }
    }

    async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<&str>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        let endpoint = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tv",
        };

        let url = format!(
            "https://api.themoviedb.org/3/search/{}?api_key={}&query={}&year={}",
            endpoint,
            self.config.api_key,
            title,
            year.unwrap_or("")
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let results: TmdbSearchResponse = response.json().await?;
            Ok(results.results.into_iter().map(|item| item.into()).collect())
        } else {
            Err(AppError::MetadataError(format!(
                "TMDB API error: {}",
                response.status()
            )))
        }
    }

    async fn get_ids(&self, tmdb_id: u32, media_type: MediaType) -> Result<MediaIds, AppError> {
        let endpoint = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tv",
        };

        let url = format!(
            "https://api.themoviedb.org/3/{}/{}/external_ids?api_key={}",
            endpoint,
            tmdb_id,
            self.config.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let ids: TmdbIdsResponse = response.json().await?;
            Ok(ids.into())
        } else {
            Err(AppError::MetadataError(format!(
                "TMDB API error: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl MetadataProvider for TmdbClient {
    fn name(&self) -> &'static str {
        "TMDB"
    }

    async fn fetch(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        let results = self.search(title, media_type, year).await?;
        if let Some(result) = results.into_iter().next() {
            let ids = self.get_ids(result.ids.tmdb.as_ref().unwrap().parse()?, media_type).await?;
            Ok(MetadataResult {
                ids,
                ..result
            })
        } else {
            Err(AppError::MetadataError("No results found".into()))
        }
    }
}

#[derive(Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbItem>,
}

#[derive(Deserialize)]
struct TmdbItem {
    id: u32,
    title: String,
    release_date: Option<String>,
    first_air_date: Option<String>,
}

#[derive(Deserialize)]
struct TmdbIdsResponse {
    imdb_id: Option<String>,
    tvdb_id: Option<u32>,
}

impl From<TmdbItem> for MetadataResult {
    fn from(item: TmdbItem) -> Self {
        let year = item.release_date.or(item.first_air_date)
            .and_then(|d| d.split('-').next().map(|y| y.to_string()));
            
        MetadataResult {
            ids: MediaIds {
                tmdb: Some(item.id.to_string()),
                ..Default::default()
            },
            title: item.title,
            year,
            media_type: MediaType::Movie, // Will be overridden
        }
    }
}

impl From<TmdbIdsResponse> for MediaIds {
    fn from(ids: TmdbIdsResponse) -> Self {
        MediaIds {
            imdb: ids.imdb_id,
            tvdb: ids.tvdb_id.map(|id| id.to_string()),
            ..Default::default()
        }
    }
}