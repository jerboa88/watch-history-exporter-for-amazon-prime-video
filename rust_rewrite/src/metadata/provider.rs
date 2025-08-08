use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{MediaType, MetadataResult};

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn name(&self) -> &'static str;
    
    async fn fetch(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<&str>,
    ) -> Result<MetadataResult, AppError>;
}

pub struct RateLimitedProvider<P: MetadataProvider> {
    provider: P,
    // Will implement rate limiting logic here
}

#[async_trait]
impl<P: MetadataProvider> MetadataProvider for RateLimitedProvider<P> {
    fn name(&self) -> &'static str {
        self.provider.name()
    }

    async fn fetch(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        // TODO: Add rate limiting
        self.provider.fetch(title, media_type, year).await
    }
}