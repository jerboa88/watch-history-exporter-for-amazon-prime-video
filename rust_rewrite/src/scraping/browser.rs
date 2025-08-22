use fantoccini::{Client, ClientBuilder};
use crate::error::AppError;
use std::time::Duration;

pub struct BrowserController {
    client: Option<Client>,
    headless: bool,
    timeout: Duration,
}

impl BrowserController {
    pub fn new(headless: bool, timeout_secs: u64) -> Self {
        Self {
            client: None,
            headless,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub async fn start(&mut self) -> Result<(), AppError> {
        let mut builder = ClientBuilder::native();
        // Note: Headless mode would need to be configured differently based on the WebDriver

        self.client = Some(
            builder
                .connect("http://localhost:4444")
                .await
                .map_err(|e| AppError::BrowserError(e.to_string()))?
        );
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), AppError> {
        if let Some(client) = self.client.take() {
            let mut client = client;
            client.close().await.map_err(|e| AppError::BrowserError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn restart(&mut self) -> Result<(), AppError> {
        self.shutdown().await?;
        self.start().await
    }

    pub fn client(&self) -> Option<&Client> {
        self.client.as_ref()
    }

    pub async fn take_screenshot(&mut self, path: &str) -> Result<(), AppError> {
        if let Some(client) = &mut self.client {
            let screenshot = client.screenshot().await
                .map_err(|e| AppError::BrowserError(e.to_string()))?;
            tokio::fs::write(path, screenshot).await
                .map_err(|e| AppError::BrowserError(e.to_string()))?;
        }
        Ok(())
    }
}