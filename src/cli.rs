use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output CSV file path
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short = 'L', long, value_name = "LEVEL", default_value = "info")]
    pub log_level: String,

    /// Run browser in headless mode
    #[arg(long)]
    pub headless: bool,

    /// Maximum number of concurrent requests
    #[arg(long, default_value = "5")]
    pub max_concurrent: usize,

    /// Timeout for browser operations (in seconds)
    #[arg(long, default_value = "30")]
    pub browser_timeout: u64,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            config: None,
            output: None,
            log_level: "info".to_string(),
            headless: true,
            max_concurrent: 5,
            browser_timeout: 30,
        }
    }
}

impl CliArgs {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Valid levels are: {}",
                self.log_level,
                valid_levels.join(", ")
            ));
        }

        // Validate max_concurrent
        if self.max_concurrent == 0 {
            return Err("max-concurrent must be greater than 0".to_string());
        }

        // Validate browser_timeout
        if self.browser_timeout == 0 {
            return Err("browser-timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}