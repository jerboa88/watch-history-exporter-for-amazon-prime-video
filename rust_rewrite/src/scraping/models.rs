use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    TvShow {
        season: Option<u32>,
        episode: Option<u32>,
        episode_title: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub raw_text: String,
    pub title: String,
    pub original_title: Option<String>,
    pub media_type: MediaType,
    pub watched_at: DateTime<Local>,
    pub is_original_language: bool,
}

impl HistoryItem {
    pub fn parse(raw_text: &str) -> Option<Self> {
        let watched_at = Self::extract_date(raw_text)?;
        let (title, original_title) = Self::extract_title(raw_text)?;
        let media_type = Self::determine_media_type(raw_text);

        Some(Self {
            raw_text: raw_text.to_string(),
            title,
            original_title: original_title.clone(),
            media_type,
            watched_at,
            is_original_language: original_title.is_none(),
        })
    }

    fn extract_date(text: &str) -> Option<DateTime<Local>> {
        use chrono::NaiveDate;
        use regex::Regex;

        // Try multiple date patterns
        let patterns = [
            r"(\w{3} \d{1,2}, \d{4})",  // "Aug 21, 2023"
            r"(\d{1,2}/\d{1,2}/\d{4})", // "08/21/2023"
            r"(\d{4}-\d{2}-\d{2})",     // "2023-08-21"
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Ok(naive_date) = NaiveDate::parse_from_str(&caps[1], "%b %d, %Y")
                        .or_else(|_| NaiveDate::parse_from_str(&caps[1], "%m/%d/%Y"))
                        .or_else(|_| NaiveDate::parse_from_str(&caps[1], "%Y-%m-%d"))
                    {
                        return Some(naive_date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap().into());
                    }
                }
            }
        }
        None
    }

    fn extract_title(text: &str) -> Option<(String, Option<String>)> {
        use regex::Regex;

        // Pattern for localized title with original in parentheses
        if let Ok(re) = Regex::new(r"^(.*?)\s*\((.*?)\)") {
            if let Some(caps) = re.captures(text) {
                return Some((caps[2].trim().to_string(), Some(caps[1].trim().to_string())));
            }
        }

        // Fallback to using whole text as title
        Some((text.trim().to_string(), None))
    }

    fn determine_media_type(text: &str) -> MediaType {
        use regex::Regex;

        // Check for TV show patterns
        let tv_patterns = [
            r"(?i)season\s+(\d+)\s+episode\s+(\d+)",
            r"(?i)s(\d+)e(\d+)",
            r"(?i)episode\s+(\d+)",
        ];

        for pattern in tv_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    return MediaType::TvShow {
                        season: caps.get(1).and_then(|m| m.as_str().parse().ok()),
                        episode: caps.get(2).and_then(|m| m.as_str().parse().ok()),
                        episode_title: None,
                    };
                }
            }
        }

        MediaType::Movie
    }
}