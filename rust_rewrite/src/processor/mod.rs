mod csv_generator;
mod history_processor;
mod progress_tracker;

use crate::{
    config::OutputConfig,
    error::AppError,
    metadata::MetadataService,
    scraping::Scraper,
};
use csv_generator::CsvGenerator;
use history_processor::HistoryProcessor;
use progress_tracker::ProgressTracker;

pub struct Processor {
    scraper: Scraper,
    metadata: MetadataService,
    csv_gen: CsvGenerator,
    progress: ProgressTracker,
}

impl Processor {
    pub fn new(
        scraper: Scraper,
        metadata: MetadataService,
        output_config: OutputConfig,
    ) -> Self {
        Self {
            scraper,
            metadata,
            csv_gen: CsvGenerator::new(output_config),
            progress: ProgressTracker::new(),
        }
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        self.progress.start();
        
        // 1. Scrape watch history
        let items = self.scraper.scrape_watch_history().await?;
        self.progress.log_scraped(items.len());
        
        // 2. Process and enrich with metadata
        let processed = HistoryProcessor::process(items, &self.metadata, &mut self.progress).await?;
        
        // 3. Generate CSV output
        self.csv_gen.generate(processed)?;
        
        self.progress.complete();
        Ok(())
    }
}