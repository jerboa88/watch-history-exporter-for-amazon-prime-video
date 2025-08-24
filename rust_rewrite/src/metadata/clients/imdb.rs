use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::{
    config::ImdbConfig,
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider, RateLimit},
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
        _year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        let _type_param = match media_type {
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

    async fn get_details(&self, imdb_id: &str) -> Result<MetadataResult, AppError> {
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
            Ok(MetadataResult {
                ids: MediaIds {
                    imdb: Some(details.id.clone()),
                    ..Default::default()
                },
                title: details.title.unwrap_or_else(|| format!("IMDB-{}", details.id)),
                year: details.year.map(|y| y.to_string()),
                media_type: MediaType::Movie, // Could be determined from details
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

    async fn search(
        &self,
        title: &str,
        _media_type: MediaType,
        _year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        let _type_param = match _media_type {
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

    async fn get_details(
        &self,
        id: &str,
        _media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        self.get_details(id).await
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
    title: Option<String>,
    year: Option<i32>,
}

impl From<ImdbItem> for MetadataResult {
    fn from(item: ImdbItem) -> Self {
        let year = item.description
            .as_ref()
            .and_then(|desc| {
                desc.split('(').nth(1)
                    .and_then(|s| s.split(')').next())
                    .map(|y| y.to_string())
            });

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