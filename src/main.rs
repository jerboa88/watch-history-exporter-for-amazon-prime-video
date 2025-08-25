use std::error::Error;

mod app;
mod cli;
mod config;
mod error;
mod metadata;
mod models;
mod scraping;
mod processor;
mod shutdown;

use app::App;
use cli::CliArgs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let cli_args = CliArgs::parse_args();

    // Validate CLI arguments
    cli_args.validate()?;

    // Initialize logging with CLI log level
    let log_level = match cli_args.log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    tracing::info!("Starting Prime Video to Simkl exporter");

    // Setup shutdown handling
    let shutdown_manager = shutdown::setup_shutdown_handler().await?;

    // Load configuration with CLI overrides
    let config = config::AppConfig::load_with_cli_args(&cli_args)?;

    // Create the application
    let mut app = App::new_with_config(config)?;

    // Run the application with shutdown handling
    tokio::select! {
        result = app.run() => {
            match result {
                Ok(()) => tracing::info!("Application completed successfully"),
                Err(e) => {
                    tracing::error!("Application error: {}", e);
                    return Err(e.into());
                }
            }
        }
        _ = shutdown_manager.wait_for_shutdown() => {
            tracing::info!("Application shutdown requested");
        }
    }

    // Perform cleanup
    tracing::info!("Performing cleanup...");
    // Add any cleanup logic here if needed

    Ok(())
}