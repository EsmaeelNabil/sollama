use sollama::{
    config::ScraperConfig,
    prompt::PromptBuilder,
    search::SearchEngine,
};
use tokio;

#[tokio::test]
async fn test_full_search_workflow() {
    let config = ScraperConfig::default();
    let search_engine = SearchEngine::new(config.clone()).unwrap();
    let query = "rust programming test";

    // Test search
    let urls = search_engine.search(query).await.unwrap();
    assert!(!urls.is_empty(), "Search should return at least one URL");

    // Test content fetching
    let contents = search_engine.fetch_all(urls).await.unwrap();
    assert!(!contents.is_empty(), "Should fetch content from at least one URL");

    // Test prompt building
    let prompt = PromptBuilder::new(query.to_string())
        .with_contents(contents)
        .build();

    assert!(prompt.contains(query), "Prompt should contain the original query");
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut config = ScraperConfig::default();
    config.rate_limit.requests_per_second = 1.0;

    let search_engine = SearchEngine::new(config).unwrap();

    let start = std::time::Instant::now();

    for _ in 0..3 {
        let _ = search_engine.search("test").await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() >= 2, "Rate limiting should space out requests");
}