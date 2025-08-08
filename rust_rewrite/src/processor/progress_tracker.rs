use std::time::Instant;
use crate::error::AppError;
use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressTracker {
    pb: ProgressBar,
    start_time: Instant,
    total_items: usize,
}

impl ProgressTracker {
    pub fn new() -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .unwrap(),
        );

        Self {
            pb,
            start_time: Instant::now(),
            total_items: 0,
        }
    }

    pub fn start(&mut self) {
        self.pb.set_message("Starting processing...");
        self.pb.enable_steady_tick(100);
    }

    pub fn log_scraped(&mut self, count: usize) {
        self.total_items = count;
        self.pb.set_message(format!(
            "Scraped {} items, starting metadata lookup...", 
            count
        ));
    }

    pub fn log_processing(&mut self, title: &str) {
        self.pb.set_message(format!(
            "Processing: {} ({} remaining)", 
            title,
            self.total_items
        ));
        self.total_items = self.total_items.saturating_sub(1);
    }

    pub fn log_processed(&mut self, count: usize) {
        self.pb.set_message(format!(
            "Processed {} items, generating CSV...", 
            count
        ));
    }

    pub fn complete(&self) {
        self.pb.finish_with_message(format!(
            "Completed in {:.2} seconds",
            self.start_time.elapsed().as_secs_f32()
        ));
    }
}