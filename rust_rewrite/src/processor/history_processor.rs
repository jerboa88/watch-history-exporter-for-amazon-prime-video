use crate::{
    error::AppError,
    metadata::{MetadataService, MediaType, MetadataResult},
    models::{WatchHistoryItem, WatchStatus},
    processor::progress_tracker::ProgressTracker,
};
use std::collections::HashMap;
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct HistoryProcessor {
    semaphore: Arc<Semaphore>,
}

impl Default for HistoryProcessor {
    fn default() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(5)), // Max 5 concurrent requests
        }
    }
}

impl HistoryProcessor {
    pub async fn process(
        items: Vec<WatchHistoryItem>,
        metadata: &MetadataService,
        progress: &mut ProgressTracker,
    ) -> Result<Vec<ProcessedItem>, AppError> {
        let processor = Self::default();
        let mut processed = Vec::with_capacity(items.len());
        let mut tv_shows: HashMap<String, WatchHistoryItem> = HashMap::new();

        // First pass: Deduplicate TV shows and process items
        for item in items {
            progress.log_processing(&item.title);

            let media_type = if item.episode.is_some() {
                MediaType::Tv
            } else {
                MediaType::Movie
            };

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

            // Process item directly without spawning
            let _permit = processor.semaphore.acquire().await?;

            // Retry logic (3 attempts)
            let mut attempts = 0;
            let mut last_error = None;

            while attempts < 3 {
                match metadata.lookup(&item.title, media_type, None).await {
                    Ok(meta) => {
                        processed.push(ProcessedItem::from_watch_history(item, meta));
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        attempts += 1;
                        if attempts < 3 {
                            tokio::time::sleep(std::time::Duration::from_secs(attempts)).await;
                        }
                    }
                }
            }

            if let Some(e) = last_error {
                if attempts >= 3 {
                    return Err(e);
                }
            }
        }

        // Process TV shows
        for (_, item) in tv_shows {
            let _permit = processor.semaphore.acquire().await?;

            // Retry logic for TV shows
            let mut attempts = 0;
            let mut last_error = None;

            while attempts < 3 {
                match metadata.lookup(&item.title, MediaType::Tv, None).await {
                    Ok(meta) => {
                        processed.push(ProcessedItem::from_watch_history(item, meta));
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        attempts += 1;
                        if attempts < 3 {
                            tokio::time::sleep(std::time::Duration::from_secs(attempts)).await;
                        }
                    }
                }
            }

            if let Some(e) = last_error {
                if attempts >= 3 {
                    return Err(e);
                }
            }
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{MetadataResult, MediaIds};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;

    struct MockMetadataService {
        call_count: AtomicUsize,
        should_fail: Mutex<bool>,
    }

    impl MockMetadataService {
        fn new() -> Self {
            Self {
                call_count: AtomicUsize::new(0),
                should_fail: Mutex::new(false),
            }
        }

        async fn set_fail(&self, fail: bool) {
            *self.should_fail.lock().await = fail;
        }
    }

    impl MockMetadataService {
        async fn lookup(
            &self,
            title: &str,
            media_type: MediaType,
            _year: Option<i32>,
        ) -> Result<MetadataResult, AppError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);

            if *self.should_fail.lock().await {
                return Err(AppError::MetadataError("Mock failure".to_string()));
            }

            Ok(MetadataResult {
                ids: MediaIds {
                    simkl: Some(format!("simkl_{}", title)),
                    tvdb: Some(format!("tvdb_{}", title)),
                    tmdb: Some(format!("tmdb_{}", title)),
                    imdb: Some(format!("imdb_{}", title)),
                    mal: Some(format!("mal_{}", title)),
                },
                title: title.to_string(),
                year: Some("2020".to_string()),
                media_type,
            })
        }
    }

    #[tokio::test]
    async fn test_deduplicates_tv_episodes() {
        let metadata = MockMetadataService::new();
        let mut progress = ProgressTracker::new();
        
        let items = vec![
            WatchHistoryItem {
                simkl_id: None,
                tvdb_id: None,
                tmdb_id: None,
                imdb_id: None,
                mal_id: None,
                media_type: MediaType::Tv,
                title: "Show A".to_string(),
                year: None,
                episode: Some("S1E1".to_string()),
                watch_status: WatchStatus::Completed,
                date: "2023-01-01".to_string(),
                rating: None,
                memo: None,
            },
            WatchHistoryItem {
                simkl_id: None,
                tvdb_id: None,
                tmdb_id: None,
                imdb_id: None,
                mal_id: None,
                media_type: MediaType::Tv,
                title: "Show A".to_string(),
                year: None,
                episode: Some("S1E2".to_string()),
                watch_status: WatchStatus::Completed,
                date: "2023-01-02".to_string(),
                rating: None,
                memo: None,
            },
        ];

        let processed = HistoryProcessor::process(items, &metadata, &mut progress)
            .await
            .unwrap();

        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].date, "2023-01-02");
    }

    #[tokio::test]
    async fn test_concurrent_processing() {
        let metadata = MockMetadataService::new();
        let mut progress = ProgressTracker::new();
        
        let items = (0..10).map(|i| WatchHistoryItem {
            simkl_id: None,
            tvdb_id: None,
            tmdb_id: None,
            imdb_id: None,
            mal_id: None,
            media_type: MediaType::Movie,
            title: format!("Movie {}", i),
            year: None,
            episode: None,
            watch_status: WatchStatus::Completed,
            date: "2023-01-01".to_string(),
            rating: None,
            memo: None,
        }).collect();

        let processed = HistoryProcessor::process(items, &metadata, &mut progress)
            .await
            .unwrap();

        assert_eq!(processed.len(), 10);
        assert_eq!(metadata.call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let metadata = MockMetadataService::new();
        metadata.set_fail(true).await;
        let mut progress = ProgressTracker::new();
        
        let items = vec![WatchHistoryItem {
            simkl_id: None,
            tvdb_id: None,
            tmdb_id: None,
            imdb_id: None,
            mal_id: None,
            media_type: MediaType::Movie,
            title: "Movie".to_string(),
            year: None,
            episode: None,
            watch_status: WatchStatus::Completed,
            date: "2023-01-01".to_string(),
            rating: None,
            memo: None,
        }];

        let result = HistoryProcessor::process(items, &metadata, &mut progress)
            .await;

        assert!(result.is_err());
    }
}