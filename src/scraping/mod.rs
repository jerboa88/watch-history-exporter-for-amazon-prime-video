pub mod models;
mod login;
mod extractor;
mod browser;
use login::{handle_login, LoginMethod};
use extractor::HistoryExtractor;
use browser::BrowserController;

use fantoccini::Client;
use crate::error::AppError;
use crate::config::AmazonConfig;
use std::time::Duration;

pub struct Scraper {
    browser: BrowserController,
    client: Option<Client>,
    config: AmazonConfig,
}

impl Scraper {
    pub async fn new(config: AmazonConfig, headless: bool) -> Result<Self, AppError> {
        let mut browser = BrowserController::new(headless, 30);
        browser.start().await?;
        let client = browser.client().cloned();

        Ok(Self {
            browser,
            client,
            config,
        })
    }

    pub async fn login(&mut self, attempt_auto_login: bool) -> Result<(), AppError> {
        let method = if attempt_auto_login {
            LoginMethod::Automated {
                email: self.config.email.clone(),
                password: self.config.password.clone(),
            }
        } else {
            LoginMethod::Manual
        };

        if let Some(client) = &mut self.client {
            handle_login(client, method).await?;
            Ok(())
        } else {
            Err(AppError::BrowserError("Browser client not initialized".into()))
        }
    }

    pub async fn scrape_watch_history(&mut self) -> Result<Vec<models::HistoryItem>, AppError> {
        const MAX_RETRIES: usize = 3;
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < MAX_RETRIES {
            match self.try_scrape().await {
                Ok(items) => return Ok(items),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < MAX_RETRIES {
                        self.restart_browser().await?;
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(||
            AppError::BrowserError("Max retries exceeded".into())
        ))
    }

    async fn try_scrape(&mut self) -> Result<Vec<models::HistoryItem>, AppError> {
        self.navigate_to_history().await?;
        if let Some(client) = &mut self.client {
            let mut extractor = HistoryExtractor::new(client);
            extractor.extract().await
        } else {
            Err(AppError::BrowserError("Browser client not initialized".into()))
        }
    }

    async fn navigate_to_history(&mut self) -> Result<(), AppError> {
        if let Some(client) = &mut self.client {
            client
                .goto("https://www.primevideo.com/settings/watch-history")
                .await
                .map_err(|e| AppError::BrowserError(e.to_string()))?;

            // Verify we reached the correct page
            let current_url = client.current_url().await
                .map_err(|e| AppError::BrowserError(e.to_string()))?;

            if !current_url.as_str().contains("watch-history") {
                return Err(AppError::BrowserError("Failed to navigate to history page".into()));
            }

            Ok(())
        } else {
            Err(AppError::BrowserError("Browser client not initialized".into()))
        }
    }

    #[allow(dead_code)]
    pub async fn take_screenshot(&mut self, path: &str) -> Result<(), AppError> {
        self.browser.take_screenshot(path).await
    }

    pub async fn restart_browser(&mut self) -> Result<(), AppError> {
        self.browser.restart().await?;
        self.client = self.browser.client().cloned();
        Ok(())
    }
}