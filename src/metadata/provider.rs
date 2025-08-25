use async_trait::async_trait;
use crate::error::AppError;
use crate::metadata::models::{MediaType, MetadataResult};

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn name(&self) -> &'static str;
    
    async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError>;
    
    async fn get_details(
        &self,
        id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError>;
}

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct TokenBucket {
    tokens: f64,
    capacity: f64,
    fill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    #[allow(dead_code)]
    fn new(capacity: f64, fill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            fill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;
        self.tokens = (self.tokens + elapsed * self.fill_rate).min(self.capacity);
    }

    fn consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn wait_time(&self, tokens: f64) -> Duration {
        let needed = tokens - self.tokens;
        if needed <= 0.0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(needed / self.fill_rate)
        }
    }
}

pub struct RateLimitedProvider<P: MetadataProvider> {
    provider: P,
    bucket: Arc<Mutex<TokenBucket>>,
}

impl<P: MetadataProvider> RateLimitedProvider<P> {
    #[allow(dead_code)]
    pub fn new(provider: P, calls: u32, per_seconds: u64) -> Self {
        let fill_rate = calls as f64 / per_seconds as f64;
        Self {
            provider,
            bucket: Arc::new(Mutex::new(TokenBucket::new(calls as f64, fill_rate))),
        }
    }
}

#[async_trait]
impl<P: MetadataProvider> MetadataProvider for RateLimitedProvider<P> {
    fn name(&self) -> &'static str {
        self.provider.name()
    }

    async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MetadataResult>, AppError> {
        // Check and consume token in a single operation
        let wait_time = {
            let mut bucket = self.bucket.lock().unwrap();
            if bucket.consume(1.0) {
                None // Can proceed immediately
            } else {
                Some(bucket.wait_time(1.0))
            }
        };

        if let Some(wait_time) = wait_time {
            sleep(wait_time).await;
            // Recursively retry after waiting
            return self.search(title, media_type, year).await;
        }

        self.provider.search(title, media_type, year).await
    }
    
    async fn get_details(
        &self,
        id: &str,
        media_type: MediaType,
    ) -> Result<MetadataResult, AppError> {
        // Check and consume token in a single operation
        let wait_time = {
            let mut bucket = self.bucket.lock().unwrap();
            if bucket.consume(1.0) {
                None // Can proceed immediately
            } else {
                Some(bucket.wait_time(1.0))
            }
        };

        if let Some(wait_time) = wait_time {
            sleep(wait_time).await;
            // Recursively retry after waiting
            return self.get_details(id, media_type).await;
        }

        self.provider.get_details(id, media_type).await
    }
}