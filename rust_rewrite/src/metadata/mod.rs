mod clients;
mod models;
mod provider;

use crate::config::{PriorityOrder, RateLimitConfig, SimklConfig, TmdbConfig, TvdbConfig, ImdbConfig, MalConfig};
use crate::error::AppError;
use clients::{simkl::SimklClient, tmdb::TmdbClient, tvdb::TvdbClient, imdb::ImdbClient, mal::MalClient};
use models::*;
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
        imdb_config: ImdbConfig,
        mal_config: MalConfig,
    ) -> Self {
        let mut providers: Vec<Box<dyn MetadataProvider>> = Vec::new();

        for service in priority_order {
            match service {
                ServiceType::Simkl => providers.push(Box::new(
                    SimklClient::new(simkl_config.clone(), rate_limits.simkl)
                )),
                ServiceType::Tmdb => providers.push(Box::new(
                    TmdbClient::new(tmdb_config.clone(), rate_limits.tmdb)
                )),
                ServiceType::Tvdb => providers.push(Box::new(
                    TvdbClient::new(tvdb_config.clone(), rate_limits.tvdb)
                )),
                ServiceType::Imdb => providers.push(Box::new(
                    ImdbClient::new(imdb_config.clone(), rate_limits.imdb)
                )),
                ServiceType::Mal => providers.push(Box::new(
                    MalClient::new(mal_config.clone(), rate_limits.mal)
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
        for provider in &self.providers {
            match provider.fetch(title, media_type, year).await {
                Ok(result) => return Ok(result),
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