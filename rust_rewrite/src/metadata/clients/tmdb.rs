use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::TmdbConfig,
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider, RateLimit},
};

#[allow(dead_code)]
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

    async fn search_internal(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        let type_param = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tv",
        };

        let mut query = vec![
            ("query".to_string(), title.to_string()),
            ("include_adult".to_string(), "false".to_string()),
        ];

        if let Some(y) = year {
            query.push(("year".to_string(), y.to_string()));
        }

        let url = format!(
            "https://api.themoviedb.org/3/search/{}?api_key={}",
            type_param,
            self.config.api_key
        );

        let response = self.client
            .get(&url)
            .query(&query)
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

    async fn get_details_internal(
        &self,
        tmdb_id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        let type_param = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tv",
        };

        let url = format!(
            "https://api.themoviedb.org/3/{}/{}?api_key={}&append_to_response=external_ids",
            type_param,
            tmdb_id,
            self.config.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let details: TmdbDetailsResponse = response.json().await?;
            Ok(details.into())
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

    async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        self.search_internal(title, media_type, year).await
    }

    async fn get_details(
        &self,
        id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        self.get_details_internal(id, media_type).await
    }
}

#[derive(serde::Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbItem>,
}

#[derive(serde::Deserialize)]
struct TmdbItem {
    id: i32,
    title: String,
    name: String,
    release_date: Option<String>,
    first_air_date: Option<String>,
    media_type: Option<String>,
}

#[derive(serde::Deserialize)]
struct TmdbDetailsResponse {
    id: i32,
    title: Option<String>,
    name: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    external_ids: TmdbExternalIds,
}

#[derive(serde::Deserialize)]
struct TmdbExternalIds {
    imdb_id: Option<String>,
    tvdb_id: Option<i32>,
}

impl From<TmdbItem> for MetadataResult {
    fn from(item: TmdbItem) -> Self {
        let title = if item.title.is_empty() { item.name } else { item.title };
        let year = item.release_date.or(item.first_air_date)
            .and_then(|d| d.split('-').next().map(|s| s.to_string()));

        MetadataResult {
            ids: MediaIds {
                tmdb: Some(item.id.to_string()),
                ..Default::default()
            },
            title,
            year,
            media_type: match item.media_type.as_deref() {
                Some("tv") => MediaType::Tv,
                Some("movie") => MediaType::Movie,
                _ => MediaType::Movie, // Default to movie if unclear
            },
        }
    }
}

impl From<TmdbDetailsResponse> for MetadataResult {
    fn from(details: TmdbDetailsResponse) -> Self {
        let has_title = details.title.is_some();
        let title = details.title.or(details.name).unwrap_or_default();
        let year = details.release_date.or(details.first_air_date)
            .and_then(|d| d.split('-').next().map(|s| s.to_string()));

        MetadataResult {
            ids: MediaIds {
                tmdb: Some(details.id.to_string()),
                imdb: details.external_ids.imdb_id,
                tvdb: details.external_ids.tvdb_id.map(|id| id.to_string()),
                ..Default::default()
            },
            title,
            year,
            media_type: if has_title {
                MediaType::Movie
            } else {
                MediaType::Tv
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serde_json::json;

    #[tokio::test]
    async fn test_search_movie() {
        let mut server = Server::new();
        let mock_response = json!({
            "results": [{
                "id": 123,
                "title": "Inception",
                "name": "",
                "release_date": "2010-07-16",
                "media_type": "movie"
            }]
        });

        let _m = server.mock("GET", "/search/movie?api_key=test&query=Inception&include_adult=false")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = TmdbClient::new(
            TmdbConfig {
                api_key: "test".to_string(),
            },
            RateLimit { calls: 10, per_seconds: 1 }
        );

        let results = client.search("Inception", MediaType::Movie, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Inception");
        assert_eq!(results[0].ids.tmdb, Some("123".to_string()));
    }

    #[tokio::test]
    async fn test_get_details() {
        let mut server = Server::new();
        let mock_response = json!({
            "id": 123,
            "title": "Inception",
            "release_date": "2010-07-16",
            "external_ids": {
                "imdb_id": "tt1375666",
                "tvdb_id": 12345
            }
        });

        let _m = server.mock("GET", "/movie/123?api_key=test&append_to_response=external_ids")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = TmdbClient::new(
            TmdbConfig {
                api_key: "test".to_string(),
            },
            RateLimit { calls: 10, per_seconds: 1 }
        );

        let result = client.get_details("123", MediaType::Movie)
            .await
            .unwrap();

        assert_eq!(result.title, "Inception");
        assert_eq!(result.ids.imdb, Some("tt1375666".to_string()));
        assert_eq!(result.ids.tvdb, Some("12345".to_string()));
    }
}