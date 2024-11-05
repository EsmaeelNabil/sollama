use std::sync::Arc;
use crate::{Result, ScraperError, ScraperConfig, ScrapedContent};
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tokio::time::sleep;
use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::Semaphore;
use tracing::{debug, error};

/// The `SearchEngine` struct is responsible for performing search operations and fetching content from URLs.
/// It uses the `reqwest` library for HTTP requests and the `scraper` library for parsing HTML.
pub struct SearchEngine {
    /// The HTTP client used for making requests.
    client: Client,
    /// The configuration settings for the scraper.
    config: ScraperConfig,
    /// The rate limiter used to control the rate of requests.
    rate_limiter: Arc<Semaphore>,
    /// The progress bar used to display progress information.
    progress: MultiProgress,
}

impl SearchEngine {
    /// Creates a new `SearchEngine` with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration settings for the scraper.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SearchEngine` instance, or an error if the client could not be created.
    pub fn new(config: ScraperConfig) -> Result<Self> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.timeout)
            .gzip(true)
            .build()
            .map_err(|e| ScraperError::RequestError(e))?;

        // Initialize rate limiter
        let rate_limiter = Arc::new(Semaphore::new(config.rate_limit.burst_size));

        Ok(Self {
            client,
            config,
            rate_limiter,
            progress: MultiProgress::new(),
        })
    }

    /// Performs a search operation and returns a list of URLs.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query.
    /// * `result_count` - The number of search results to return.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of URLs, or an error if the search fails.
    pub async fn search(&self, query: &str, result_count: &str) -> Result<Vec<String>> {
        let search_pb = self.progress.add(ProgressBar::new_spinner());
        search_pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        search_pb.set_message(format!("Searching for '{}'...", query));

        sleep(Duration::from_secs(1)).await;

        let url = format!(
            "https://www.google.com/search?q={}&hl=en&num={}",
            urlencoding::encode(query), result_count
        );

        debug!("Search URL: {}", url);

        let response = self.client
            .get(&url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .send()
            .await
            .map_err(|e| {
                ScraperError::RequestError(e)
            })?;

        let status = response.status();
        debug!("Response status: {}", status);

        search_pb.set_message("Processing search results...");
        let html = response.text().await?;

        let document = Html::parse_document(&html);
        self.extract_urls(&document)
    }

    /// Fetches content from all the given URLs.
    ///
    /// # Arguments
    ///
    /// * `urls` - A vector of URLs to fetch content from.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ScrapedContent`, or an error if the fetch fails.
    pub async fn fetch_all(&self, urls: Vec<String>) -> Result<Vec<ScrapedContent>> {
        let total_urls = urls.len();

        let fetch_pb = self.progress.add(ProgressBar::new_spinner());

        fetch_pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        fetch_pb.set_message(format!("Fetching pages...{}", urls.len()));

        let fetches = stream::iter(urls)
            .map(|url| {
                let rate_limiter = self.rate_limiter.clone();
                let fetch_pb = fetch_pb.clone();
                async move {
                    // Acquire rate limit permit
                    let _permit = rate_limiter.acquire().await.expect("Rate limiter closed");
                    let delay = Duration::from_secs_f32(1.0 / self.config.rate_limit.requests_per_second);
                    sleep(delay).await;

                    fetch_pb.set_message(format!("Fetching {}", url));

                    match self.fetch_content(&url).await {
                        Ok(content) => {
                            Some(content)
                        }
                        Err(_e) => {
                            None
                        }
                    }
                }
            })
            .buffer_unordered(self.config.concurrent_requests)
            .collect::<Vec<_>>()
            .await;

        let contents: Vec<ScrapedContent> = fetches.into_iter()
            .filter_map(|x| x)
            .collect();

        let success_count = contents.len();
        fetch_pb.finish_with_message(format!(
            "Completed: {} of {} pages scraped successfully",
            success_count,
            total_urls
        ));
        Ok(contents)
    }

    /// Fetches content from a single URL with retries.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch content from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ScrapedContent`, or an error if the fetch fails.
    async fn fetch_content(&self, url: &str) -> Result<ScrapedContent> {
        debug!("Fetching content from: {}", url);

        let mut retries = 0;
        let mut last_error = None;

        while retries < self.config.max_retries {
            match self.try_fetch_content(url).await {
                Ok(content) => return Ok(content),
                Err(e) => {
                    retries += 1;
                    last_error = Some(e);
                    if retries < self.config.max_retries {
                        let delay = Duration::from_secs(2u64.pow(retries));
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ScraperError::ExtractionError("Max retries exceeded".to_string())
        }))
    }

    /// Attempts to fetch content from a single URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch content from.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ScrapedContent`, or an error if the fetch fails.
    async fn try_fetch_content(&self, url: &str) -> Result<ScrapedContent> {
        let response = self.client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .send()
            .await?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let content = self.extract_text(&document)?;

        Ok(ScrapedContent {
            url: url.to_string(),
            content,
            metadata: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Extracts URLs from the HTML document.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed HTML document.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of URLs, or an error if no URLs are found.
    fn extract_urls(&self, document: &Html) -> Result<Vec<String>> {
        // Try multiple selector patterns that Google might use
        let selector_patterns = [
            "div.g div.yuRUbf > a",           // Common pattern
            "div.tF2Cxc > div.yuRUbf > a",    // Alternative pattern
            "div.g a[href]",                  // More general pattern
            "div[class='g'] a[ping]",         // Another common pattern
            "div.rc > a",                     // Legacy pattern
            "div.r > a",                      // Legacy pattern
            "a[data-ved]",                    // Links with data-ved attribute
        ];

        let mut all_urls = Vec::new();

        for pattern in selector_patterns {
            debug!("Trying selector pattern: {}", pattern);

            if let Ok(selector) = Selector::parse(pattern) {
                let urls: Vec<String> = document
                    .select(&selector)
                    .filter_map(|link| {
                        let href = link.value().attr("href")?;
                        debug!("Found raw URL: {}", href);

                        if let Some(clean_url) = self.clean_google_url(href) {
                            if self.is_valid_url(&clean_url) {
                                debug!("Valid URL found: {}", clean_url);
                                Some(clean_url)
                            } else {
                                debug!("Invalid URL: {}", clean_url);
                                None
                            }
                        } else {
                            debug!("Could not clean URL: {}", href);
                            None
                        }
                    })
                    .collect();

                all_urls.extend(urls);
            }
        }

        // Remove duplicates
        all_urls.sort();
        all_urls.dedup();

        if all_urls.is_empty() {
            error!("No valid URLs found in the response");
        } else {
            for (i, url) in all_urls.iter().enumerate() {
                debug!("URL {}: {}", i + 1, url);
            }
        }

        Ok(all_urls)
    }

    /// Cleans a Google redirect URL to extract the actual URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The Google redirect URL.
    ///
    /// # Returns
    ///
    /// An `Option` containing the cleaned URL, or `None` if the URL could not be cleaned.
    fn clean_google_url(&self, url: &str) -> Option<String> {
        debug!("Cleaning URL: {}", url);

        // Handle Google redirect URLs
        if url.starts_with("/url?") || url.contains("/url?") {
            let url_str = url.replace("/url?", "");
            if let Some(query) = url_str.split('&').find(|&q| q.starts_with("q=")) {
                let clean = query.replace("q=", "");
                let decoded = urlencoding::decode(&clean).ok()?.into_owned();
                debug!("Cleaned redirect URL: {}", decoded);
                return Some(decoded);
            }
        }

        // Handle absolute URLs
        if url.starts_with("http") {
            debug!("Found absolute URL: {}", url);
            return Some(url.to_string());
        }

        debug!("URL could not be cleaned: {}", url);
        None
    }

    /// Checks if a URL is valid.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    ///
    /// # Returns
    ///
    /// `true` if the URL is valid, `false` otherwise.
    fn is_valid_url(&self, url: &str) -> bool {
        // Invalid patterns
        let invalid_patterns = [
            "google.com/search",
            "google.com/url",
            "google.com/imgres",
            "accounts.google",
            "webcache.googleusercontent",
            "/preferences",
            "/settings",
            "/advanced_search",
            "/setprefs",
            "javascript:",
        ];

        let is_valid = url.starts_with("https://") &&
            !invalid_patterns.iter().any(|&pattern| url.contains(pattern)) &&
            !url.contains("&");

        if is_valid {
            debug!("URL is valid: {}", url);
        } else {
            debug!("URL is invalid: {}", url);
        }

        is_valid
    }

    /// Extracts text content from the HTML document using predefined selectors.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed HTML document.
    ///
    /// # Returns
    ///
    /// A `Result` containing the extracted text content, or an error if no content is found.
    fn extract_text(&self, document: &Html) -> Result<String> {
        let selectors = [
            "article p, article li",
            "div.content p, div.content li",
            "main p, main li",
            ".documentation-content",
            "div.markdown-body",
            "div.mw-parser-output p",
            "p, li",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                let content: String = document
                    .select(&selector)
                    .map(|element| element.text().collect::<Vec<_>>().join(" "))
                    .collect::<Vec<_>>()
                    .join("\n");

                if !content.trim().is_empty() {
                    return Ok(content.trim().to_string());
                }
            }
        }

        Err(ScraperError::ExtractionError("No content found".to_string()))
    }
}