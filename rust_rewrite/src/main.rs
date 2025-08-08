use std::error::Error;

mod config;
mod error;
mod models;

use config::AppConfig;
use error::AppError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::load()?;

    tracing::info!("Starting Prime Video to Simkl exporter");
    tracing::debug!("Loaded configuration: {:?}", config);

    Ok(())
}