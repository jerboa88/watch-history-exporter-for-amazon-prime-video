use crate::{
    error::AppError,
    metadata::{MetadataService, MediaType},
    scraping::WatchHistoryItem,
    processor::progress_tracker::ProgressTracker,
};
use std::collections::HashMap;

pub struct HistoryProcessor;

impl HistoryProcessor {
    pub async fn process(
        items: Vec<WatchHistoryItem>,
        metadata: &MetadataService,
        progress: &mut ProgressTracker,
    ) -> Result<Vec<ProcessedItem>, AppError> {
        let mut processed = Vec::with_capacity(items.len());
        let mut tv_shows = HashMap::new();

        for item in items {
            progress.log_processing(&item.title);
            
            let media_type = if item.episode.is_some() {
                MediaType::Tv
            } else {
                MediaType::Movie
            };

            // For TV shows, only keep the latest episode
            if media_type == MediaType::Tv {
                if let Some(existing) = tv_shows.get_mut(&item.title) {
                    if item.date > existing.date {
                        *existing = item;
                    }
                    continue;
                } else {
                    tv_shows.insert(item.title.clone(), item);
                    continue;
                }
            }

            // Process movie or the latest TV episode
            let metadata = metadata.lookup(&item.title, media_type, None).await?;
            processed.push(ProcessedItem::from_watch_history(item, metadata));
        }

        // Process the kept TV episodes
        for (_, item) in tv_shows {
            let metadata = metadata.lookup(&item.title, MediaType::Tv, None).await?;
            processed.push(ProcessedItem::from_watch_history(item, metadata));
        }

        progress.log_processed(processed.len());
        Ok(processed)
    }
}

pub struct ProcessedItem {
    pub title: String,
    pub date: String,
    pub media_type: MediaType,
    pub metadata: MetadataResult,
    pub episode: Option<String>,
}

impl ProcessedItem {
    pub fn from_watch_history(item: WatchHistoryItem, metadata: MetadataResult) -> Self {
        Self {
            title: item.title,
            date: item.date,
            media_type: if item.episode.is_some() {
                MediaType::Tv
            } else {
                MediaType::Movie
            },
            metadata,
            episode: item.episode,
        }
    }
}