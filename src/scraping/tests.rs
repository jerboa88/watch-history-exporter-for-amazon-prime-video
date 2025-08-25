use super::*;
use fantoccini::MockClient;
use mockall::predicate::*;

#[tokio::test]
async fn test_history_item_parsing() {
    let test_cases = vec![
        (
            "The Boys (Aug 21, 2023)",
            ("The Boys", None, MediaType::Movie),
        ),
        (
            "Localized (Original) (Sep 1, 2023)",
            ("Original", Some("Localized"), MediaType::Movie),
        ),
        (
            "Show S01E02 (Jul 15, 2023)",
            ("Show S01E02", None, 
             MediaType::TvShow { 
                 season: Some(1), 
                 episode: Some(2), 
                 episode_title: None 
             }),
        ),
    ];

    for (input, (exp_title, exp_orig, exp_type)) in test_cases {
        let item = HistoryItem::parse(input).unwrap();
        assert_eq!(item.title, exp_title);
        assert_eq!(item.original_title, exp_orig);
        assert!(matches!(item.media_type, exp_type));
    }
}

#[tokio::test]
async fn test_scraper_retry_logic() {
    let mut mock = MockClient::new();
    
    // First attempt fails
    mock.expect_goto()
        .times(1)
        .returning(|_| Err(fantoccini::error::CmdError::NotW3C("failed".to_string())));
    
    // Second attempt succeeds
    mock.expect_goto()
        .times(1)
        .returning(|_| Ok(()));
    
    mock.expect_current_url()
        .times(1)
        .returning(|| Ok("https://www.primevideo.com/settings/watch-history".into()));

    let mut scraper = Scraper {
        browser: BrowserController::new(false, 30),
        client: Some(mock),
        config: AmazonConfig::default(),
    };

    let result = scraper.scrape_watch_history().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_navigation_verification() {
    let mut mock = MockClient::new();
    mock.expect_goto()
        .times(1)
        .returning(|_| Ok(()));
    
    // Return wrong URL
    mock.expect_current_url()
        .times(1)
        .returning(|| Ok("https://www.primevideo.com/signin".into()));

    let mut scraper = Scraper {
        browser: BrowserController::new(false, 30),
        client: Some(mock),
        config: AmazonConfig::default(),
    };

    let result = scraper.navigate_to_history().await;
    assert!(result.is_err());
}