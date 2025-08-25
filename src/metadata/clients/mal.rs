use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::MalConfig,
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider, RateLimit},
};

#[allow(dead_code)]
pub struct MalClient {
    client: Client,
    config: MalConfig,
    rate_limit: RateLimit,
    access_token: Option<String>,
}

impl MalClient {
    pub fn new(config: MalConfig, rate_limit: RateLimit) -> Self {
        Self {
            client: Client::new(),
            config,
            rate_limit,
            access_token: None,
        }
    }

    async fn authenticate(&mut self) -> Result<(), AppError> {
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("grant_type", &"client_credentials".to_string())
        ];

        let response = self.client
            .post("https://myanimelist.net/v1/oauth2/token")
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let auth: MalAuthResponse = response.json().await?;
            self.access_token = Some(auth.access_token);
            Ok(())
        } else {
            Err(AppError::AuthError("MAL authentication failed".into()))
        }
    }

    async fn search_internal(
        &mut self,
        title: &str,
    ) -> Result<Vec<MetadataResult>, AppError> {
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        let url = format!(
            "https://api.myanimelist.net/v2/anime?q={}&limit=5&fields=id,title,start_date",
            title
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token.as_ref().unwrap()))
            .send()
            .await?;

        if response.status().is_success() {
            let results: MalSearchResponse = response.json().await?;
            Ok(results.data.into_iter().map(|item| item.into()).collect())
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            Box::pin(self.search_internal(title)).await
        } else {
            Err(AppError::MetadataError(format!(
                "MAL API error: {}",
                response.status()
            )))
        }
    }

    async fn get_details_internal(&mut self, mal_id: u32) -> Result<MetadataResult, AppError> {
        if self.access_token.is_none() {
            self.authenticate().await?;
        }

        let url = format!(
            "https://api.myanimelist.net/v2/anime/{}?fields=id,title,start_date",
            mal_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token.as_ref().unwrap()))
            .send()
            .await?;

        if response.status().is_success() {
            let item: MalItemResponse = response.json().await?;
            let year = item.start_date
                .as_ref()
                .and_then(|d| d.split('-').next())
                .map(|y| y.to_string());

            Ok(MetadataResult {
                ids: MediaIds {
                    mal: Some(item.id.to_string()),
                    ..Default::default()
                },
                title: item.title,
                year,
                media_type: MediaType::Tv,
            })
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            Box::pin(self.get_details_internal(mal_id)).await
        } else {
            Err(AppError::MetadataError(format!(
                "MAL API error: {}",
                response.status()
            )))
        }
    }
}

#[async_trait]
impl MetadataProvider for MalClient {
    fn name(&self) -> &'static str {
        "MyAnimeList"
    }

    async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        _year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        if media_type != MediaType::Tv {
            return Ok(vec![]); // MAL only supports anime
        }

        // Need mutable self for auth
        let mut this = unsafe { std::ptr::read(self) };
        let result = this.search_internal(title).await;
        std::mem::forget(this);
        result
    }

    async fn get_details(
        &self,
        id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        if media_type != MediaType::Tv {
            return Err(AppError::MetadataError("MAL only supports anime".into()));
        }

        let mal_id = id.parse::<u32>()?;
        // Need mutable self for auth
        let mut this = unsafe { std::ptr::read(self) };
        let result = this.get_details_internal(mal_id).await;
        std::mem::forget(this);
        result
    }
}

#[derive(serde::Deserialize)]
struct MalAuthResponse {
    access_token: String,
}

#[derive(serde::Deserialize)]
struct MalSearchResponse {
    data: Vec<MalItem>,
}

#[derive(serde::Deserialize)]
struct MalItem {
    node: MalItemDetails,
}

#[derive(serde::Deserialize)]
struct MalItemDetails {
    id: u32,
    title: String,
    start_date: Option<String>,
}

#[derive(serde::Deserialize)]
struct MalItemResponse {
    id: u32,
    title: String,
    start_date: Option<String>,
}

impl From<MalItem> for MetadataResult {
    fn from(item: MalItem) -> Self {
        let year = item.node.start_date
            .as_ref()
            .and_then(|d| d.split('-').next())
            .map(|y| y.to_string());

        MetadataResult {
            ids: MediaIds {
                mal: Some(item.node.id.to_string()),
                ..Default::default()
            },
            title: item.node.title,
            year,
            media_type: MediaType::Tv,
        }
    }
}