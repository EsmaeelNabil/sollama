use std::time::Duration;
use thiserror::Error;

pub mod config;
pub mod prompt;
pub mod scraper;
pub mod search;
pub mod types;
pub mod llm;

// Re-export commonly used types
pub use config::ScraperConfig;
pub use types::{ScrapedContent, SearchResult};

/// The `ScraperError` enum represents various errors that can occur in the scraper application.
#[derive(Error, Debug)]
pub enum ScraperError {
    /// Represents an error that occurs during an HTTP request.
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Represents an error that occurs when the rate limit is exceeded.
    #[error("Rate limit exceeded")]
    RateLimitError,
    /// Represents an error that occurs during content extraction.
    #[error("Content extraction failed: {0}")]
    ExtractionError(String),
    /// Represents an error that occurs during LLM processing.
    #[error("LLM processing failed: {0}")]
    LLMError(String),
    /// Represents an error that occurs during a search operation.
    #[error("Search failed: {0}")]
    SearchError(String),
}

/// A type alias for `Result` with the `ScraperError` error type.
pub type Result<T> = std::result::Result<T, ScraperError>;

// Constants

/// The default timeout duration for HTTP requests.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
/// The default number of concurrent requests.
pub const DEFAULT_CONCURRENT_REQUESTS: usize = 5;
/// The default maximum number of retries for failed requests.
pub const DEFAULT_MAX_RETRIES: u32 = 3;