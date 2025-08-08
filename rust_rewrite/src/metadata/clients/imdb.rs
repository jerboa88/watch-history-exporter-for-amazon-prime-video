use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::{ImdbConfig, RateLimit},
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider},
};

pub struct ImdbClient {
    client: Client,
    config: ImdbConfig,
    rate_limit: RateLimit,
}

impl ImdbClient {
    pub fn new(config: ImdbConfig, rate_limit: RateLimit) -> Self {
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
    ) -> Result<Vec<MetadataResult>, AppError> {
        let type_param = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tvSeries",
        };

        let url = format!(
            "https://imdb-api.com/API/Search/{}/{}",
            self.config.api_key,
            title
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let results: ImdbSearchResponse = response.json().await?;
            Ok(results.results.into_iter().map(|item| item.into()).collect())
        } else {
            Err(AppError::MetadataError(format!(
                "IMDB API error: {}",
                response.status()
            )))
        }
    }

    async fn get_details(&self, imdb_id: &str) -> Result<MediaIds, AppError> {
        let url = format!(
            "https://imdb-api.com/API/Title/{}/{}",
            self.config.api_key,
            imdb_id
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let details: ImdbDetailsResponse = response.json().await?;
            Ok(MediaIds {
                imdb: Some(details.id),
                ..Default::default()
            })
        } else {
            Err(AppError::MetadataError(format!(
                "IMDB API error: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl MetadataProvider for ImdbClient {
    fn name(&self) -> &'static str {
        "IMDB"
    }

    async fn fetch(
        &self,
        title: &str,
        media_type: MediaType,
        _year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        let results = self.search(title, media_type).await?;
        if let Some(result) = results.into_iter().next() {
            let imdb_id = result.ids.imdb.as_ref().unwrap();
            let ids = self.get_details(imdb_id).await?;
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
struct ImdbSearchResponse {
    results: Vec<ImdbItem>,
}

#[derive(Deserialize)]
struct ImdbItem {
    id: String,
    title: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct ImdbDetailsResponse {
    id: String,
}

impl From<ImdbItem> for MetadataResult {
    fn from(item: ImdbItem) -> Self {
        let year = item.description
            .and_then(|desc| desc.split('(').nth(1))
            .and_then(|s| s.split(')').next())
            .map(|y| y.to_string());

        MetadataResult {
            ids: MediaIds {
                imdb: Some(item.id),
                ..Default::default()
            },
            title: item.title,
            year,
            media_type: MediaType::Movie, // Will be overridden
        }
    }
}