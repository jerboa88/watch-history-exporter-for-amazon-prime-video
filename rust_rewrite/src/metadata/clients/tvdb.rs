use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::TvdbConfig,
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider, RateLimit},
};

pub struct TvdbClient {
    client: Client,
    config: TvdbConfig,
    rate_limit: RateLimit,
    access_token: Option<String>,
}

impl TvdbClient {
    pub fn new(config: TvdbConfig, rate_limit: RateLimit) -> Self {
        Self {
            client: Client::new(),
            config,
            rate_limit,
            access_token: None,
        }
    }

    async fn authenticate(&mut self) -> Result<(), AppError> {
        let auth = serde_json::json!({
            "apikey": self.config.api_key
        });

        let response = self.client
            .post("https://api.thetvdb.com/login")
            .json(&auth)
            .send()
            .await?;

        if response.status().is_success() {
            let auth: TvdbAuthResponse = response.json().await?;
            self.access_token = Some(auth.token);
            Ok(())
        } else {
            Err(AppError::AuthError("TVDB authentication failed".into()))
        }
    }

    async fn search_internal(
        &mut self,
        title: &str,
        media_type: MediaType,
    ) -> Result<Vec<MetadataResult>, AppError> {
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        let url = format!(
            "https://api.thetvdb.com/search/series?name={}",
            title
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token.as_ref().unwrap()))
            .send()
            .await?;

        if response.status().is_success() {
            let results: TvdbSearchResponse = response.json().await?;
            Ok(results.data.into_iter().map(|item| item.into()).collect())
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            Box::pin(self.search_internal(title, media_type)).await
        } else {
            Err(AppError::MetadataError(format!(
                "TVDB API error: {}",
                response.status()
            )))
        }
    }

    async fn get_details_internal(
        &mut self,
        tvdb_id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        let url = format!(
            "https://api.thetvdb.com/series/{}",
            tvdb_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token.as_ref().unwrap()))
            .send()
            .await?;

        if response.status().is_success() {
            let details: TvdbDetailsResponse = response.json().await?;
            Ok(details.data.into())
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            Box::pin(self.get_details_internal(tvdb_id, media_type)).await
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

    async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        _year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        // Need mutable self for auth
        let mut this = unsafe { std::ptr::read(self) };
        let result = this.search_internal(title, media_type).await;
        std::mem::forget(this);
        result
    }

    async fn get_details(
        &self,
        id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        // Need mutable self for auth
        let mut this = unsafe { std::ptr::read(self) };
        let result = this.get_details_internal(id, media_type).await;
        std::mem::forget(this);
        result
    }
}

#[derive(serde::Deserialize)]
struct TvdbAuthResponse {
    token: String,
}

#[derive(serde::Deserialize)]
struct TvdbSearchResponse {
    data: Vec<TvdbSearchItem>,
}

#[derive(serde::Deserialize)]
struct TvdbSearchItem {
    id: i32,
    seriesName: String,
    firstAired: Option<String>,
}

#[derive(serde::Deserialize)]
struct TvdbDetailsResponse {
    data: TvdbDetailsItem,
}

#[derive(serde::Deserialize)]
struct TvdbDetailsItem {
    id: i32,
    seriesName: String,
    firstAired: Option<String>,
    imdbId: Option<String>,
}

impl From<TvdbSearchItem> for MetadataResult {
    fn from(item: TvdbSearchItem) -> Self {
        let year = item.firstAired
            .as_ref()
            .and_then(|d| d.split('-').next())
            .map(|y| y.to_string());

        MetadataResult {
            ids: MediaIds {
                tvdb: Some(item.id.to_string()),
                ..Default::default()
            },
            title: item.seriesName,
            year,
            media_type: MediaType::Tv,
        }
    }
}

impl From<TvdbDetailsItem> for MetadataResult {
    fn from(item: TvdbDetailsItem) -> Self {
        let year = item.firstAired
            .as_ref()
            .and_then(|d| d.split('-').next())
            .map(|y| y.to_string());

        MetadataResult {
            ids: MediaIds {
                tvdb: Some(item.id.to_string()),
                imdb: item.imdbId,
                ..Default::default()
            },
            title: item.seriesName,
            year,
            media_type: MediaType::Tv,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Server};
    use serde_json::json;

    #[tokio::test]
    async fn test_search() {
        let mut server = Server::new();
        
        // Mock auth endpoint
        let _m_auth = mock("POST", "/login")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"token": "test_token"}).to_string())
            .create();

        // Mock search endpoint
        let mock_response = json!({
            "data": [{
                "id": 123,
                "seriesName": "Breaking Bad",
                "firstAired": "2008-01-20"
            }]
        });

        let _m_search = mock("GET", "/search/series?name=Breaking%20Bad")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("Authorization", "Bearer test_token")
            .with_body(mock_response.to_string())
            .create();

        let client = TvdbClient::new(
            TvdbConfig {
                api_key: "test".to_string(),
            },
            RateLimit { calls: 10, per_seconds: 1 }
        );

        let results = client.search("Breaking Bad", MediaType::Tv, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Breaking Bad");
        assert_eq!(results[0].ids.tvdb, Some("123".to_string()));
    }

    #[tokio::test]
    async fn test_get_details() {
        let mut server = Server::new();
        
        // Mock auth endpoint
        let _m_auth = mock("POST", "/login")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"token": "test_token"}).to_string())
            .create();

        // Mock details endpoint
        let mock_response = json!({
            "data": {
                "id": 123,
                "seriesName": "Breaking Bad",
                "firstAired": "2008-01-20",
                "imdbId": "tt0903747"
            }
        });

        let _m_details = mock("GET", "/series/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("Authorization", "Bearer test_token")
            .with_body(mock_response.to_string())
            .create();

        let client = TvdbClient::new(
            TvdbConfig {
                api_key: "test".to_string(),
            },
            RateLimit { calls: 10, per_seconds: 1 }
        );

        let result = client.get_details("123", MediaType::Tv)
            .await
            .unwrap();

        assert_eq!(result.title, "Breaking Bad");
        assert_eq!(result.ids.imdb, Some("tt0903747".to_string()));
    }
}