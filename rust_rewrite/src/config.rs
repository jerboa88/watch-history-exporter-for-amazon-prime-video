use serde::Deserialize;
use config::Config;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub simkl: SimklConfig,
    pub tmdb: TmdbConfig,
    pub tvdb: TvdbConfig,
    pub mal: MalConfig,
    pub amazon: AmazonConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize)]
pub struct SimklConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct TmdbConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TvdbConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct MalConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct AmazonConfig {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: PathBuf,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;

        config.try_deserialize()
    }
}