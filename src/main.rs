use std::time::Instant;
use tracing::{error};
use sollama::{
    config::ScraperConfig,
    prompt::PromptBuilder,
    search::SearchEngine,
    llm::LLMProcessor,
    Result,
};

/// The main entry point of the application.
///
/// This function initializes logging, loads the configuration, processes command line arguments,
/// performs a search, fetches content from URLs, and processes the content using a Language Model (LLM).
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = ScraperConfig::default();

    // Get query from command line arguments
    let search_query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "rust programming".to_string());

    let query = std::env::args()
        .nth(2)
        .unwrap_or_else(|| format!("based on the content provided what is : {}", search_query));

    let results_count = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "5".to_string());

    let model = std::env::args()
        .nth(4)
        .unwrap_or_else(|| "llama3.2:latest".to_string());

    let start_time = Instant::now();

    // Initialize search engine
    let search_engine = SearchEngine::new(config.clone())?;

    // Perform search and content gathering
    let urls = search_engine.search(&search_query, &results_count).await?;

    if urls.len() == 0 {
        error!("No URLs found for the query: {}", query);
        return Ok(());
    }

    // Fetch content from all URLs
    let contents = search_engine.fetch_all(urls.clone()).await?;

    // Process with LLM
    let llm_processor = LLMProcessor::new(config.llm_config);
    let prompt = PromptBuilder::new(query.clone())
        .with_contents(contents.clone())
        .build();

    match llm_processor.process(&prompt, &model).await {
        Ok(summary) => {
            let elapsed = start_time.elapsed();

            println!("{}", format!("\n=== Search Results Summary ===\n {}\n", &urls.join("\n")));
            println!("\n=== Search Results Summary ===");
            println!("Search Query: {}", search_query);
            println!("Query: {}", query);
            println!("Processing time: {:.2?}", elapsed);
            println!("Pages analyzed: {}", contents.len());
            println!("\nSummary:\n{}", summary);
        }
        Err(e) => {
            error!("Failed to process with LLM: {}", e);
        }
    }

    Ok(())
}