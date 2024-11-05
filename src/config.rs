use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The `ScraperConfig` struct holds the configuration settings for the scraper application.
/// It includes settings for concurrent requests, timeout, retries, user agent, rate limiting, and LLM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    /// The number of concurrent requests allowed.
    pub concurrent_requests: usize,
    /// The timeout duration for HTTP requests.
    pub timeout: Duration,
    /// The maximum number of retries for failed requests.
    pub max_retries: u32,
    /// The user agent string to be used in HTTP requests.
    pub user_agent: String,
    /// The rate limit settings for the scraper.
    pub rate_limit: RateLimit,
    /// The configuration settings for the Language Model (LLM).
    pub llm_config: LLMConfig,
}

/// The `RateLimit` struct holds the rate limiting settings for the scraper.
/// It includes the number of requests per second and the burst size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// The number of requests allowed per second.
    pub requests_per_second: f32,
    /// The burst size for rate limiting.
    pub burst_size: usize,
}

/// The `LLMConfig` struct holds the configuration settings for the Language Model (LLM).
/// It includes the endpoint URL, temperature, and maximum number of tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// The endpoint URL for the LLM API.
    pub endpoint: String,
    /// The temperature setting for the LLM, controlling the randomness of the output.
    pub temperature: f32,
    /// The maximum number of tokens allowed in the LLM response.
    pub max_tokens: u32,
}

impl Default for ScraperConfig {
    /// Provides default values for the `ScraperConfig` struct.
    ///
    /// # Returns
    ///
    /// A `ScraperConfig` instance with default settings.
    fn default() -> Self {
        Self {
            concurrent_requests: crate::DEFAULT_CONCURRENT_REQUESTS,
            timeout: crate::DEFAULT_TIMEOUT,
            max_retries: crate::DEFAULT_MAX_RETRIES,
            user_agent: String::from("Mozilla/5.0 (compatible; RustBot/1.0)"),
            rate_limit: RateLimit {
                requests_per_second: 2.0,
                burst_size: 5,
            },
            llm_config: LLMConfig {
                endpoint: String::from("http://localhost:11434/api/generate"),
                temperature: 0.1,
                max_tokens: 2048,
            },
        }
    }
}