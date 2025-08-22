pub mod csv_generator;
pub mod history_processor;
pub mod progress_tracker;

// Re-export the main structs for easier access
pub use csv_generator::CsvGenerator;
pub use history_processor::HistoryProcessor;
pub use progress_tracker::ProgressTracker;

use crate::{
    config::OutputConfig,
    error::AppError,
    metadata::MetadataService,
    scraping::{Scraper, models as scraping_models},
    models,
};
// Individual imports removed - using re-exports instead

pub struct Processor {
    scraper: Scraper,
    csv_gen: CsvGenerator,
    progress: ProgressTracker,
}

impl Processor {
    pub fn new(
        scraper: Scraper,
        output_config: OutputConfig,
    ) -> Self {
        Self {
            scraper,
            csv_gen: CsvGenerator::new(output_config),
            progress: ProgressTracker::new(),
        }
    }

    pub async fn run(&mut self, metadata: &MetadataService) -> Result<(), AppError> {
        self.progress.start("Starting processing");

        // 1. Scrape watch history
        let items = self.scraper.scrape_watch_history().await?;
        self.progress.log_scraped(items.len());

        // 2. Convert HistoryItem to WatchHistoryItem for processing
        let watch_items: Vec<models::WatchHistoryItem> = items.into_iter().map(|item| {
            // Convert scraping MediaType to models MediaType
            let media_type = match item.media_type {
                scraping_models::MediaType::Movie => models::MediaType::Movie,
                scraping_models::MediaType::TvShow { .. } => models::MediaType::Tv,
            };

            // Extract episode info from scraping MediaType
            let episode = match item.media_type {
                scraping_models::MediaType::Movie => None,
                scraping_models::MediaType::TvShow { season, episode, episode_title } => {
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

            models::WatchHistoryItem {
                simkl_id: None, // Will be filled by metadata service
                tvdb_id: None,
                tmdb_id: None,
                imdb_id: None,
                mal_id: None,
                media_type,
                title: item.title,
                year: None, // Could be extracted from watched_at if needed
                episode,
                watch_status: models::WatchStatus::Completed,
                date: item.watched_at.format("%Y-%m-%d").to_string(),
                rating: None,
                memo: None,
            }
        }).collect();

        // 3. Process and enrich with metadata
        let processed = HistoryProcessor::process(watch_items, &metadata, &mut self.progress).await?;

        // 4. Generate CSV output
        self.csv_gen.generate(processed)?;

        self.progress.complete("Processing complete");
        Ok(())
    }
}