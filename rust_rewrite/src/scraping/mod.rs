mod login;
mod extractor;
use login::{handle_login, LoginMethod};
use extractor::HistoryExtractor;

use fantoccini::{Client, Locator};
use crate::error::AppError;
use crate::config::AmazonConfig;

pub struct Scraper {
    client: Client,
    config: AmazonConfig,
}

impl Scraper {
    pub async fn new(config: AmazonConfig) -> Result<Self, AppError> {
        let client = Client::native()
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;
            
        Ok(Self { client, config })
    }

    pub async fn login(&self, attempt_auto_login: bool) -> Result<(), AppError> {
        let method = if attempt_auto_login {
            LoginMethod::Automated {
                email: self.config.email.clone(),
                password: self.config.password.clone(),
            }
        } else {
            LoginMethod::Manual
        };

        handle_login(&self.client, method).await
    }

    pub async fn scrape_watch_history(&self) -> Result<Vec<String>, AppError> {
        self.navigate_to_history().await?;
        let extractor = HistoryExtractor::new(&self.client);
        extractor.extract().await
    }

    async fn navigate_to_history(&self) -> Result<(), AppError> {
        self.client
            .goto("https://www.primevideo.com/settings/watch-history")
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;
        Ok(())
    }
}