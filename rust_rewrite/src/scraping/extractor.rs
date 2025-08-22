use fantoccini::{Client, Locator, elements::Element};
use crate::error::AppError;
use crate::scraping::models::HistoryItem;
use std::time::Duration;

pub struct HistoryExtractor<'a> {
    client: &'a mut Client,
    max_attempts: usize,
    scroll_delay: Duration,
}

impl<'a> HistoryExtractor<'a> {
    pub fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            max_attempts: 100,
            scroll_delay: Duration::from_secs(2),
        }
    }

    pub async fn extract(&mut self) -> Result<Vec<HistoryItem>, AppError> {
        self.load_all_items().await?;
        self.parse_history().await
    }

    async fn load_all_items(&mut self) -> Result<(), AppError> {
        let mut previous_height = 0;
        let mut current_height = 1;
        let mut attempts = 0;

        while previous_height != current_height && attempts < self.max_attempts {
            previous_height = current_height;
            attempts += 1;

            // Scroll to bottom with error recovery
            self.scroll_to_bottom().await?;

            // Wait for loading with timeout
            tokio::time::sleep(self.scroll_delay).await;

            // Check for new height with retry logic
            current_height = self.get_scroll_height().await?;
        }

        Ok(())
    }

    async fn scroll_to_bottom(&mut self) -> Result<(), AppError> {
        for attempts in 0..3 { // Retry up to 3 times
            match self.client
                .execute(
                    "window.scrollTo(0, document.body.scrollHeight)",
                    vec![],
                )
                .await
            {
                Ok(_) => return Ok(()),
                Err(e) if attempts == 2 => return Err(AppError::BrowserError(e.to_string())),
                Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
            }
        }
        Ok(())
    }

    async fn get_scroll_height(&mut self) -> Result<usize, AppError> {
        for attempts in 0..3 { // Retry up to 3 times
            match self.client
                .execute("return document.body.scrollHeight", vec![])
                .await
            {
                Ok(height) => return Ok(height.as_i64().unwrap_or(0) as usize),
                Err(e) if attempts == 2 => return Err(AppError::BrowserError(e.to_string())),
                Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
            }
        }
        Ok(0)
    }

    async fn parse_history(&mut self) -> Result<Vec<HistoryItem>, AppError> {
        let mut history = Vec::new();
        let mut attempts = 0;
        const MAX_PARSE_ATTEMPTS: usize = 3;

        while attempts < MAX_PARSE_ATTEMPTS {
            match self.try_parse_history_items().await {
                Ok(items) => return Ok(items),
                Err(e) if attempts == MAX_PARSE_ATTEMPTS - 1 => return Err(e),
                Err(_) => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Ok(history)
    }

    async fn try_parse_history_items(&mut self) -> Result<Vec<HistoryItem>, AppError> {
        let items = self.client
            .find_all(Locator::Css(
                "div[data-automation-id='activity-history-items'] li",
            ))
            .await
            .map_err(|e| AppError::BrowserError(e.to_string()))?;

        let mut history = Vec::with_capacity(items.len());
        for mut item in items {
            match self.extract_item_text(&mut item).await {
                Ok(text) => {
                    if let Some(parsed) = HistoryItem::parse(&text) {
                        history.push(parsed);
                    } else {
                        log::warn!("Failed to parse history item: {}", text);
                    }
                },
                Err(e) => log::warn!("Failed to extract item text: {}", e),
            }
        }

        if history.is_empty() {
            Err(AppError::ParseError("No history items found".into()))
        } else {
            Ok(history)
        }
    }

    async fn extract_item_text(&mut self, item: &mut Element) -> Result<String, AppError> {
        item.text()
            .await
            .map_err(|e| AppError::ParseError(format!("Failed to get item text: {}", e)))
    }
}