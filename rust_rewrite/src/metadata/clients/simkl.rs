use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::SimklConfig,
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider, RateLimit},
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

    async fn search_internal(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        let type_param = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "show",
        };

        let mut query = vec![
            ("q".to_string(), title.to_string()),
            ("type".to_string(), type_param.to_string()),
        ];

        if let Some(y) = year {
            query.push(("year".to_string(), y.to_string()));
        }

        let response = self.client
            .get("https://api.simkl.com/search")
            .header("Authorization", format!("Bearer {}", self.config.client_secret))
            .header("simkl-api-key", &self.config.client_id)
            .query(&query)
            .send()
            .await?;

        if response.status().is_success() {
            let results: Vec<SimklSearchItem> = response.json().await?;
            Ok(results.into_iter().map(|item| item.into()).collect())
        } else {
            Err(AppError::MetadataError(format!(
                "Simkl API error: {}",
                response.status()
            )))
        }
    }

    async fn get_details_internal(
        &self,
        simkl_id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        let type_param = match media_type {
            MediaType::Movie => "movies",
            MediaType::Tv => "shows",
        };

        let url = format!(
            "https://api.simkl.com/{}/{}?extended=full",
            type_param,
            simkl_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.client_secret))
            .header("simkl-api-key", &self.config.client_id)
            .send()
            .await?;

        if response.status().is_success() {
            let details: SimklDetailsResponse = response.json().await?;
            Ok(details.into())
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
struct SimklSearchItem {
    title: String,
    year: Option<String>,
    ids: SimklIds,
}

#[derive(serde::Deserialize)]
struct SimklIds {
    simkl: String,
    imdb: Option<String>,
    tmdb: Option<String>,
    tvdb: Option<String>,
}

#[derive(serde::Deserialize)]
struct SimklDetailsResponse {
    title: String,
    year: Option<String>,
    ids: SimklIds,
}

impl From<SimklSearchItem> for MetadataResult {
    fn from(item: SimklSearchItem) -> Self {
        MetadataResult {
            ids: MediaIds {
                simkl: Some(item.ids.simkl),
                imdb: item.ids.imdb,
                tmdb: item.ids.tmdb,
                tvdb: item.ids.tvdb,
                ..Default::default()
            },
            title: item.title,
            year: item.year,
            media_type: MediaType::Movie, // Will be overridden
        }
    }
}

impl From<SimklDetailsResponse> for MetadataResult {
    fn from(details: SimklDetailsResponse) -> Self {
        MetadataResult {
            ids: MediaIds {
                simkl: Some(details.ids.simkl),
                imdb: details.ids.imdb,
                tmdb: details.ids.tmdb,
                tvdb: details.ids.tvdb,
                ..Default::default()
            },
            title: details.title,
            year: details.year,
            media_type: MediaType::Movie, // Will be overridden
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Server};
    use serde_json::json;

    #[tokio::test]
    async fn test_search_movie() {
        let mut server = Server::new();
        let mock_response = json!([{
            "title": "Inception",
            "year": "2010",
            "ids": {
                "simkl": "123",
                "imdb": "tt1375666",
                "tmdb": "12345"
            }
        }]);

        let _m = mock("GET", "/search?q=Inception&type=movie")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = SimklClient::new(
            SimklConfig {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
            },
            RateLimit { calls: 10, per_seconds: 1 }
        );

        let results = client.search("Inception", MediaType::Movie, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Inception");
        assert_eq!(results[0].ids.imdb, Some("tt1375666".to_string()));
    }

    #[tokio::test]
    async fn test_get_details() {
        let mut server = Server::new();
        let mock_response = json!({
            "title": "Inception",
            "year": "2010",
            "ids": {
                "simkl": "123",
                "imdb": "tt1375666",
                "tmdb": "12345"
            }
        });

        let _m = mock("GET", "/movies/123?extended=full")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = SimklClient::new(
            SimklConfig {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
            },
            RateLimit { calls: 10, per_seconds: 1 }
        );

        let result = client.get_details("123", MediaType::Movie)
            .await
            .unwrap();

        assert_eq!(result.title, "Inception");
        assert_eq!(result.ids.imdb, Some("tt1375666".to_string()));
    }
}