use serde::{Deserialize, Serialize};
use config::Config;
use std::path::PathBuf;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct AppConfig {
    pub simkl: SimklConfig,
    pub tmdb: TmdbConfig,
    pub tvdb: TvdbConfig,
    pub imdb: ImdbConfig,
    pub mal: MalConfig,
    pub amazon: AmazonConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct SimklConfig {
    #[validate(length(min = 1, message = "Client ID cannot be empty"))]
    pub client_id: String,
    #[validate(length(min = 1, message = "Client secret cannot be empty"))]
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct TmdbConfig {
    #[validate(length(min = 1, message = "API key cannot be empty"))]
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct TvdbConfig {
    #[validate(length(min = 1, message = "API key cannot be empty"))]
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct MalConfig {
    #[validate(length(min = 1, message = "Client ID cannot be empty"))]
    pub client_id: String,
    #[validate(length(min = 1, message = "Client secret cannot be empty"))]
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ImdbConfig {
    #[validate(length(min = 1, message = "API key cannot be empty"))]
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct AmazonConfig {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
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

    pub fn load_with_cli_args(cli_args: &crate::cli::CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        let mut builder = Config::builder()
            .add_source(config::File::with_name("config").required(false));

        // Override with CLI arguments if provided
        if let Some(config_path) = &cli_args.config {
            builder = builder.add_source(config::File::with_name(config_path.to_str().unwrap()));
        }

        let mut config = builder.build()?;

        // Override specific values from CLI args
        if let Some(output_path) = &cli_args.output {
            config.set("output.path", output_path.to_str().unwrap())?;
        }

        let app_config: AppConfig = config.try_deserialize()?;

        // Validate the configuration
        app_config.validate().map_err(|e: validator::ValidationErrors| -> Box<dyn std::error::Error> {
            format!("Configuration validation failed: {}", e).into()
        })?;

        Ok(app_config)
    }

    pub fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validator::Validate::validate(self)
    }
}