use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::{TvdbConfig, RateLimit},
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider},
};

pub struct TvdbClient {
    client: Client,
    config: TvdbConfig,
    rate_limit: RateLimit,
    token: Option<String>,
}

impl TvdbClient {
    pub fn new(config: TvdbConfig, rate_limit: RateLimit) -> Self {
        Self {
            client: Client::new(),
            config,
            rate_limit,
            token: None,
        }
    }

    async fn authenticate(&mut self) -> Result<(), AppError> {
        let response = self.client
            .post("https://api.thetvdb.com/login")
            .json(&serde_json::json!({
                "apikey": self.config.api_key
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth: TvdbAuthResponse = response.json().await?;
            self.token = Some(auth.token);
            Ok(())
        } else {
            Err(AppError::AuthError("TVDB authentication failed".into()))
        }
    }

    async fn search(
        &mut self,
        title: &str,
        media_type: MediaType,
    ) -> Result<Vec<MetadataResult>, AppError> {
        if self.token.is_none() {
            self.authenticate().await?;
        }

        let endpoint = match media_type {
            MediaType::Movie => "movies",
            MediaType::Tv => "series",
        };

        let response = self.client
            .get(&format!("https://api.thetvdb.com/search/{}?name={}", endpoint, title))
            .header("Authorization", format!("Bearer {}", self.token.as_ref().unwrap()))
            .send()
            .await?;

        if response.status().is_success() {
            let results: TvdbSearchResponse = response.json().await?;
            Ok(results.data.into_iter().map(|item| item.into()).collect())
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            self.search(title, media_type).await
        } else {
            Err(AppError::MetadataError(format!(
                "TVDB API error: {}",
                response.status()
            )))
        }
    }

    async fn get_ids(&mut self, tvdb_id: u32) -> Result<MediaIds, AppError> {
        if self.token.is_none() {
            self.authenticate().await?;
        }

        let response = self.client
            .get(&format!("https://api.thetvdb.com/series/{}", tvdb_id))
            .header("Authorization", format!("Bearer {}", self.token.as_ref().unwrap()))
            .send()
            .await?;

        if response.status().is_success() {
            let item: TvdbItemResponse = response.json().await?;
            Ok(MediaIds {
                tvdb: Some(tvdb_id.to_string()),
                imdb: item.imdbId,
                ..Default::default()
            })
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            self.get_ids(tvdb_id).await
        } else {
            Err(AppError::MetadataError(format!(
                "TVDB API error: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl MetadataProvider for TvdbClient {
    fn name(&self) -> &'static str {
        "TVDB"
    }

    async fn fetch(
        &mut self,
        title: &str,
        media_type: MediaType,
        _year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        let results = self.search(title, media_type).await?;
        if let Some(result) = results.into_iter().next() {
            let tvdb_id = result.ids.tvdb.as_ref().unwrap().parse()?;
            let ids = self.get_ids(tvdb_id).await?;
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
struct TvdbAuthResponse {
    token: String,
}

#[derive(Deserialize)]
struct TvdbSearchResponse {
    data: Vec<TvdbItem>,
}

#[derive(Deserialize)]
struct TvdbItem {
    id: u32,
    seriesName: Option<String>,
    name: Option<String>,
    firstAired: Option<String>,
}

#[derive(Deserialize)]
struct TvdbItemResponse {
    imdbId: Option<String>,
}

impl From<TvdbItem> for MetadataResult {
    fn from(item: TvdbItem) -> Self {
        let title = item.seriesName.or(item.name).unwrap_or_default();
        let year = item.firstAired.and_then(|d| d.split('-').next().map(|y| y.to_string()));
        
        MetadataResult {
            ids: MediaIds {
                tvdb: Some(item.id.to_string()),
                ..Default::default()
            },
            title,
            year,
            media_type: MediaType::Tv, // TVDB is primarily for TV shows
        }
    }
}