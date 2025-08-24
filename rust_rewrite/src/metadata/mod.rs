mod clients;
mod models;
mod provider;

pub use models::{ServiceType, MediaType, MetadataResult, MediaIds, RateLimitConfig, RateLimit, PriorityOrder};

// Internal imports needed for implementation
use crate::config::{SimklConfig, TmdbConfig, TvdbConfig, MalConfig};
use crate::error::AppError;
use clients::{SimklClient, TmdbClient, TvdbClient, ImdbClient, MalClient};
use provider::MetadataProvider;

pub struct MetadataService {
    providers: Vec<Box<dyn MetadataProvider>>,
}

impl MetadataService {
    pub fn new(
        priority_order: PriorityOrder, 
        rate_limits: RateLimitConfig,
        simkl_config: SimklConfig,
        tmdb_config: TmdbConfig,
        tvdb_config: TvdbConfig,
        imdb_config: crate::config::ImdbConfig,
        mal_config: MalConfig,
    ) -> Self {
        let mut providers: Vec<Box<dyn MetadataProvider>> = Vec::new();

        for service in priority_order {
            match service {
                ServiceType::Simkl => providers.push(Box::new(
                    SimklClient::new(simkl_config.clone(), rate_limits.simkl.clone())
                )),
                ServiceType::Tmdb => providers.push(Box::new(
                    TmdbClient::new(tmdb_config.clone(), rate_limits.tmdb.clone())
                )),
                ServiceType::Tvdb => providers.push(Box::new(
                    TvdbClient::new(tvdb_config.clone(), rate_limits.tvdb.clone())
                )),
                ServiceType::Imdb => providers.push(Box::new(
                    ImdbClient::new(imdb_config.clone(), rate_limits.imdb.clone())
                )),
                ServiceType::Mal => providers.push(Box::new(
                    MalClient::new(mal_config.clone(), rate_limits.mal.clone())
                )),
            }
        }

        Self { providers }
    }

    pub async fn lookup(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<&str>,
    ) -> Result<MetadataResult, AppError> {
        let year_int = year.and_then(|y| y.parse().ok());
        for provider in &self.providers {
            match provider.search(title, media_type, year_int).await {
                Ok(results) => {
                    if let Some(result) = results.into_iter().next() {
                        return Ok(result);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Metadata lookup failed on {}: {}",
                        provider.name(),
                        e
                    );
                    continue;
                }
            }
        }
        Err(AppError::MetadataError("All providers failed".into()))
    }
}