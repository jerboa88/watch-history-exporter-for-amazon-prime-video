use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::scraping::Scraper;
use crate::processor::{CsvGenerator, ProgressTracker};
use crate::processor::history_processor::{HistoryProcessor, ProcessedItem};
use crate::scraping::models::HistoryItem;
use crate::metadata::MetadataService;

pub struct App {
    config: AppConfig,
    progress: Arc<Mutex<ProgressTracker>>,
    scraper: Option<Scraper>,
    generator: CsvGenerator,
}

impl App {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, AppError> {
        let config = AppConfig::load()?;
        Self::new_with_config(config)
    }

    pub fn new_with_config(config: AppConfig) -> Result<Self, AppError> {
        let progress = Arc::new(Mutex::new(ProgressTracker::new()));
        let generator = CsvGenerator::new(config.output.clone());

        Ok(Self {
            config,
            progress,
            scraper: None,
            generator,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialize_browser().await?;
        self.login().await?;
        let items = self.scrape_history().await?;
        let processed = self.process_items(items).await?;
        self.generate_output(processed).await?;
        Ok(())
    }

    async fn initialize_browser(&mut self) -> Result<(), AppError> {
        {
            let mut progress = self.progress.lock().await;
            progress.start("Initializing browser");
        }

        self.scraper = Some(Scraper::new(self.config.amazon.clone(), true).await?);
        Ok(())
    }

    async fn login(&mut self) -> Result<(), AppError> {
        {
            let mut progress = self.progress.lock().await;
            progress.update("Logging in");
        }

        if let Some(scraper) = &mut self.scraper {
            scraper.login(true).await?;
        }
        Ok(())
    }

    async fn scrape_history(&mut self) -> Result<Vec<HistoryItem>, AppError> {
        {
            let mut progress = self.progress.lock().await;
            progress.update("Scraping watch history");
        }

        if let Some(scraper) = &mut self.scraper {
            let items = scraper.scrape_watch_history().await?;
            {
                let progress = self.progress.lock().await;
                progress.complete("Scraping complete");
            }
            Ok(items)
        } else {
            Err(AppError::BROWSER_NOT_INITIALIZED)
        }
    }

    async fn process_items(&mut self, items: Vec<HistoryItem>) -> Result<Vec<ProcessedItem>, AppError> {
        {
            let mut progress = self.progress.lock().await;
            progress.start("Processing data");
        }

        // Convert HistoryItem to WatchHistoryItem for processing
        let watch_items: Vec<crate::models::WatchHistoryItem> = items.into_iter().map(|item| {
            // Convert scraping MediaType to models MediaType
            let media_type = match item.media_type {
                crate::scraping::models::MediaType::Movie => crate::models::MediaType::Movie,
                crate::scraping::models::MediaType::TvShow { .. } => crate::models::MediaType::Tv,
            };

            // Extract episode info from scraping MediaType
            let episode = match item.media_type {
                crate::scraping::models::MediaType::Movie => None,
                crate::scraping::models::MediaType::TvShow { season, episode, episode_title } => {
                    let mut episode_str = String::new();
                    if let Some(s) = season {
                        episode_str.push_str(&format!("S{:02}", s));
                    }
                    if let Some(e) = episode {
                        if !episode_str.is_empty() {
                            episode_str.push_str(&format!("E{:02}", e));
                        } else {
                            episode_str.push_str(&format!("E{:02}", e));
                        }
                    }
                    if let Some(title) = episode_title {
                        if !episode_str.is_empty() {
                            episode_str.push_str(&format!(" - {}", title));
                        } else {
                            episode_str = title;
                        }
                    }
                    Some(episode_str)
                }
            };

            crate::models::WatchHistoryItem {
                simkl_id: None, // Will be filled by metadata service
                tvdb_id: None,
                tmdb_id: None,
                imdb_id: None,
                mal_id: None,
                media_type,
                title: item.title,
                year: None, // Could be extracted from watched_at if needed
                episode,
                watch_status: crate::models::WatchStatus::Completed,
                date: item.watched_at.format("%Y-%m-%d").to_string(),
                rating: None,
                memo: None,
            }
        }).collect();

        let mut progress_tracker = ProgressTracker::new();

        // Create default rate limits
        let rate_limits = crate::metadata::RateLimitConfig {
            simkl: crate::metadata::RateLimit { calls: 1000, per_seconds: 3600 },
            tmdb: crate::metadata::RateLimit { calls: 1000, per_seconds: 3600 },
            tvdb: crate::metadata::RateLimit { calls: 1000, per_seconds: 3600 },
            imdb: crate::metadata::RateLimit { calls: 1000, per_seconds: 3600 },
            mal: crate::metadata::RateLimit { calls: 1000, per_seconds: 3600 },
        };

        let metadata_service = MetadataService::new(
            vec![], // Empty priority order for now
            rate_limits,
            self.config.simkl.clone(),
            self.config.tmdb.clone(),
            self.config.tvdb.clone(),
            self.config.imdb.clone(),
            self.config.mal.clone(),
        );
        let processed = HistoryProcessor::process(watch_items, &metadata_service, &mut progress_tracker).await?;

        {
            let progress = self.progress.lock().await;
            progress.complete("Processing complete");
        }
        Ok(processed)
    }

    async fn generate_output(&mut self, items: Vec<ProcessedItem>) -> Result<(), AppError> {
        {
            let mut progress = self.progress.lock().await;
            progress.start("Generating CSV output");
        }
        self.generator.generate(items)?;
        {
            let progress = self.progress.lock().await;
            progress.complete("CSV generated successfully");
        }
        Ok(())
    }
}

impl AppError {
    pub const BROWSER_NOT_INITIALIZED: AppError = AppError::BrowserError(String::new());
}