use fantoccini::{Client, Locator};
use crate::error::AppError;

pub struct HistoryExtractor<'a> {
    client: &'a Client,
}

impl<'a> HistoryExtractor<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn extract(&self) -> Result<Vec<String>, AppError> {
        self.load_all_items().await?;
        self.parse_history().await
    }

    async fn load_all_items(&self) -> Result<(), AppError> {
        let mut previous_height = 0;
        let mut current_height = 1;
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100;

        while previous_height != current_height && attempts < MAX_ATTEMPTS {
            previous_height = current_height;
            attempts += 1;

            // Scroll to bottom
            self.client
                .execute(
                    "window.scrollTo(0, document.body.scrollHeight)",
                    vec![],
                )
                .await
                .map_err(|e| AppError::BrowserError(e.to_string()))?;

            // Wait for loading
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            // Check for new height
            current_height = self
                .client
                .execute("return document.body.scrollHeight", vec![])
                .await
                .map_err(|e| AppError::BrowserError(e.to_string()))?
                .as_i64()
                .unwrap_or(0) as usize;
        }

        Ok(())
    }

    async fn parse_history(&self) -> Result<Vec<String>, AppError> {
        let items = self
            .client
            .find_all(Locator::Css(
                "div[data-automation-id='activity-history-items'] li",
            ))
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        let mut history = Vec::new();
        for item in items {
            if let Ok(text) = item.text().await {
                history.push(text);
            }
        }

        Ok(history)
    }
}