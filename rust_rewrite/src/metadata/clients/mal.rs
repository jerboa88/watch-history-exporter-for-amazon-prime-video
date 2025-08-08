use async_trait::async_trait;
use reqwest::Client;
use crate::{
    config::{MalConfig, RateLimit},
    error::AppError,
    metadata::{MediaType, MetadataResult, MediaIds, MetadataProvider},
};

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

    async fn search(
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
            self.search(title).await
        } else {
            Err(AppError::MetadataError(format!(
                "MAL API error: {}",
                response.status()
            )))
        }
    }

    async fn get_details(&mut self, mal_id: u32) -> Result<MediaIds, AppError> {
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
            Ok(MediaIds {
                mal: Some(item.id.to_string()),
                ..Default::default()
            })
        } else if response.status() == 401 {
            // Token expired, retry with new auth
            self.authenticate().await?;
            self.get_details(mal_id).await
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

    async fn fetch(
        &mut self,
        title: &str,
        media_type: MediaType,
        _year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        if media_type != MediaType::Tv {
            return Err(AppError::MetadataError("MAL only supports anime".into()));
        }

        let results = self.search(title).await?;
        if let Some(result) = results.into_iter().next() {
            let mal_id = result.ids.mal.as_ref().unwrap().parse()?;
            let ids = self.get_details(mal_id).await?;
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
struct MalAuthResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct MalSearchResponse {
    data: Vec<MalItem>,
}

#[derive(Deserialize)]
struct MalItem {
    node: MalItemDetails,
}

#[derive(Deserialize)]
struct MalItemDetails {
    id: u32,
    title: String,
    start_date: Option<String>,
}

#[derive(Deserialize)]
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