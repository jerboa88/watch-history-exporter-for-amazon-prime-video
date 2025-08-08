use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::{SimklConfig, RateLimit},
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider},
};

pub struct SimklClient {
    client: Client,
    config: SimklConfig,
    rate_limit: RateLimit,
}

impl SimklClient {
    pub fn new(config: SimklConfig, rate_limit: RateLimit) -> Self {
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
            MediaType::Movie => "search/movie",
            MediaType::Tv => "search/tv",
        };

        let url = format!(
            "https://api.simkl.com/{endpoint}?q={}&year={}",
            title,
            year.unwrap_or("")
        );

        let response = self.client
            .get(&url)
            .header("simkl-api-key", &self.config.client_id)
            .send()
            .await?;

        if response.status().is_success() {
            let results: Vec<SimklItem> = response.json().await?;
            Ok(results.into_iter().map(|item| {
                let mut result: MetadataResult = item.into();
                result.media_type = media_type;
                result
            }).collect())
        } else {
            Err(AppError::MetadataError(format!(
                "Simkl API error: {}",
                response.status()
            )))
        }
    }

    async fn get_ids(&self, simkl_id: &str, media_type: MediaType) -> Result<MediaIds, AppError> {
        let endpoint = match media_type {
            MediaType::Movie => "movies",
            MediaType::Tv => "tv",
        };

        let url = format!(
            "https://api.simkl.com/{endpoint}/{simkl_id}?extended=full"
        );

        let response = self.client
            .get(&url)
            .header("simkl-api-key", &self.config.client_id)
            .send()
            .await?;

        if response.status().is_success() {
            let item: SimklItem = response.json().await?;
            Ok(item.into())
        } else {
            Err(AppError::MetadataError(format!(
                "Simkl API error: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl MetadataProvider for SimklClient {
    fn name(&self) -> &'static str {
        "Simkl"
    }

    async fn fetch(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        let results = self.search(title, media_type, year).await?;
        results.into_iter().next().ok_or_else(|| {
            AppError::MetadataError("No results found".into())
        })
    }
}

#[derive(Deserialize)]
struct SimklItem {
    ids: SimklIds,
    title: String,
    year: Option<String>,
}

#[derive(Deserialize)]
struct SimklIds {
    simkl: Option<String>,
    tvdb: Option<String>,
    tmdb: Option<String>,
    imdb: Option<String>,
    mal: Option<String>,
}

impl From<SimklItem> for MetadataResult {
    fn from(item: SimklItem) -> Self {
        MetadataResult {
            ids: item.ids.into(),
            title: item.title,
            year: item.year,
            media_type: MediaType::Movie, // Will be overridden in search results
        }
    }
}

impl From<SimklIds> for MediaIds {
    fn from(ids: SimklIds) -> Self {
        MediaIds {
            simkl: ids.simkl,
            tvdb: ids.tvdb,
            tmdb: ids.tmdb,
            imdb: ids.imdb,
            mal: ids.mal,
        }
    }
}