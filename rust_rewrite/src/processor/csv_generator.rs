use crate::{
    config::OutputConfig,
    error::AppError,
    metadata::MediaType,
    processor::history_processor::ProcessedItem,
};
use csv::Writer;
use std::{fs::File, path::Path};

pub struct CsvGenerator {
    output_path: String,
}

impl CsvGenerator {
    pub fn new(config: OutputConfig) -> Self {
        Self {
            output_path: config.path.to_string_lossy().to_string(),
        }
    }

    pub fn generate(&self, items: Vec<ProcessedItem>) -> Result<(), AppError> {
        let path = Path::new(&self.output_path);
        let file = File::create(path)?;
        let mut wtr = Writer::from_writer(file);

        // Write header
        wtr.write_record(&[
            "simkl_id", "TVDB_ID", "TMDB", "IMDB_ID", "MAL_ID",
            "Type", "Title", "Year", "LastEpWatched", "Watchlist",
            "WatchedDate", "Rating", "Memo"
        ])?;

        // Write each record
        for item in items {
            let ids = item.metadata.ids;
            let last_ep = item.episode.unwrap_or_default();
            let watch_status = match item.media_type {
                MediaType::Movie => "completed",
                MediaType::Tv => if last_ep.is_empty() { "completed" } else { "watching" },
            };

            wtr.write_record(&[
                ids.simkl.unwrap_or_default(),
                ids.tvdb.unwrap_or_default(),
                ids.tmdb.unwrap_or_default(),
                ids.imdb.unwrap_or_default(),
                ids.mal.unwrap_or_default(),
                match item.media_type {
                    MediaType::Movie => "movie".to_string(),
                    MediaType::Tv => "tv".to_string(),
                },
                item.title,
                item.metadata.year.unwrap_or_default(),
                last_ep,
                watch_status.to_string(),
                item.date,
                "".to_string(), // Rating (empty)
                "".to_string(), // Memo (empty)
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }
}