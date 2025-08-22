use std::time::Instant;
use tokio::time::Duration;
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

    pub fn start(&mut self, message: &str) {
        self.pb.set_message(message.to_string());
        self.pb.enable_steady_tick(Duration::from_millis(100));
    }

    pub fn update(&mut self, message: &str) {
        self.pb.set_message(message.to_string());
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

    pub fn complete(&self, message: &str) {
        self.pb.finish_with_message(format!(
            "{} in {:.2} seconds",
            message,
            self.start_time.elapsed().as_secs_f32()
        ));
    }
}

impl Clone for ProgressTracker {
    fn clone(&self) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .unwrap(),
        );
        // Progress tracker Clone implementation is complete

        Self {
            pb,
            start_time: self.start_time,
            total_items: self.total_items,
        }
    }
}