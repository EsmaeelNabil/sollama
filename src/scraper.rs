use crate::{Result, ScrapedContent, ScraperError};
use scraper::{Html, Selector};
use std::collections::HashMap;
use tracing::instrument;

/// The `ContentScraper` struct is responsible for extracting content and metadata from HTML documents.
/// It uses CSS selectors to identify the relevant parts of the document.
pub struct ContentScraper {
    /// A list of CSS selectors used to extract the main content from the HTML document.
    selectors: Vec<Selector>,
    /// A map of metadata keys to CSS selectors used to extract metadata from the HTML document.
    metadata_selectors: HashMap<String, Selector>,
}

impl Default for ContentScraper {
    /// Provides default values for the `ContentScraper` struct.
    ///
    /// # Returns
    ///
    /// A `ContentScraper` instance with default selectors.
    fn default() -> Self {
        let default_selectors = [
            "article p, article li",
            "div.content p, div.content li",
            "main p, main li",
            ".documentation-content",
            "div.markdown-body",
            "div.mw-parser-output p",
            "p, li",
        ];

        let metadata_selectors = [
            ("title", "title, h1.title, .article-title"),
            ("description", "meta[name='description']"),
            ("keywords", "meta[name='keywords']"),
            ("author", "meta[name='author'], .author"),
            ("date", "meta[name='date'], .date, time"),
        ];

        Self::new(default_selectors, metadata_selectors)
    }
}

impl ContentScraper {
    /// Creates a new `ContentScraper` with the given content and metadata selectors.
    ///
    /// # Arguments
    ///
    /// * `content_selectors` - An iterator of CSS selectors for extracting the main content.
    /// * `metadata_selectors` - An iterator of tuples containing metadata keys and their corresponding CSS selectors.
    ///
    /// # Returns
    ///
    /// A new instance of `ContentScraper`.
    pub fn new(
        content_selectors: impl IntoIterator<Item = impl AsRef<str>>,
        metadata_selectors: impl IntoIterator<Item = (impl Into<String>, impl AsRef<str>)>,
    ) -> Self {
        let selectors = content_selectors
            .into_iter()
            .filter_map(|s| Selector::parse(s.as_ref()).ok())
            .collect();

        let metadata_selectors = metadata_selectors
            .into_iter()
            .filter_map(|(key, sel)| {
                Selector::parse(sel.as_ref())
                    .ok()
                    .map(|selector| (key.into(), selector))
            })
            .collect();

        Self {
            selectors,
            metadata_selectors,
        }
    }

    /// Extracts the main content and metadata from the given HTML string.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML string to be parsed.
    /// * `url` - The URL of the HTML document.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ScrapedContent` with the extracted content and metadata, or an error if the extraction fails.
    #[instrument(skip(self, html), fields(html_length = html.len()))]
    pub fn extract(&self, html: &str, url: &str) -> Result<ScrapedContent> {
        let document = Html::parse_document(html);

        let content = self.extract_content(&document)?;
        let metadata = self.extract_metadata(&document);

        Ok(ScrapedContent {
            url: url.to_string(),
            content,
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Extracts the main content from the HTML document using the configured selectors.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed HTML document.
    ///
    /// # Returns
    ///
    /// A `Result` containing the extracted content as a string, or an error if no content is found.
    fn extract_content(&self, document: &Html) -> Result<String> {
        for selector in &self.selectors {
            let content = self.extract_text_by_selector(document, selector);
            if !content.is_empty() {
                return Ok(self.clean_text(&content));
            }
        }

        Err(ScraperError::ExtractionError(
            "No content found with available selectors".to_string(),
        ))
    }

    /// Extracts metadata from the HTML document using the configured selectors.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed HTML document.
    ///
    /// # Returns
    ///
    /// A `HashMap` containing the extracted metadata.
    fn extract_metadata(&self, document: &Html) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        for (key, selector) in &self.metadata_selectors {
            if let Some(value) = self.extract_metadata_value(document, selector) {
                metadata.insert(key.clone(), value);
            }
        }

        metadata
    }

    /// Extracts a metadata value from the HTML document using the given selector.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed HTML document.
    /// * `selector` - The CSS selector used to extract the metadata value.
    ///
    /// # Returns
    ///
    /// An `Option` containing the extracted metadata value, or `None` if no value is found.
    fn extract_metadata_value(&self, document: &Html, selector: &Selector) -> Option<String> {
        document
            .select(selector)
            .next()
            .and_then(|element| {
                // First try content attribute (for meta tags)
                if let Some(content) = element.value().attr("content") {
                    return Some(content.to_string());
                }

                // Then try text content
                let text = element.text().collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            })
    }

    /// Extracts text content from the HTML document using the given selector.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed HTML document.
    /// * `selector` - The CSS selector used to extract the text content.
    ///
    /// # Returns
    ///
    /// A string containing the extracted text content.
    fn extract_text_by_selector(&self, document: &Html, selector: &Selector) -> String {
        document
            .select(selector)
            .map(|element| {
                element.text().collect::<Vec<_>>().join(" ")
            })
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// Cleans the extracted text by removing non-ASCII characters and normalizing whitespace.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to be cleaned.
    ///
    /// # Returns
    ///
    /// The cleaned text.
    fn clean_text(&self, text: &str) -> String {
        text.chars()
            .filter(|&c| c.is_ascii() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the content extraction functionality of the `ContentScraper`.
    #[test]
    fn test_content_extraction() {
        let html = r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="Test description">
                    <meta name="author" content="Test Author">
                </head>
                <body>
                    <article>
                        <h1>Test Article</h1>
                        <p>This is a test paragraph.</p>
                        <p>This is another paragraph.</p>
                    </article>
                </body>
            </html>
        "#;

        let scraper = ContentScraper::default();
        let result = scraper.extract(html, "https://example.com").unwrap();

        assert!(result.content.contains("This is a test paragraph"));
        assert!(result.content.contains("This is another paragraph"));
        assert_eq!(result.metadata.get("title").unwrap(), "Test Page");
        assert_eq!(result.metadata.get("description").unwrap(), "Test description");
        assert_eq!(result.metadata.get("author").unwrap(), "Test Author");
    }

    /// Tests the content extraction functionality with custom selectors.
    #[test]
    fn test_custom_selectors() {
        let html = r#"
            <div class="custom-content">
                <span class="special">Special content</span>
            </div>
        "#;

        let scraper = ContentScraper::new(
            vec![".custom-content .special"],
            vec![("custom", ".special")],
        );

        let result = scraper.extract(html, "https://example.com").unwrap();

        assert!(result.content.contains("Special content"));
        assert_eq!(result.metadata.get("custom").unwrap(), "Special content");
    }

    /// Tests the content extraction functionality when no content is found.
    #[test]
    fn test_empty_content() {
        let html = "<html><body></body></html>";
        let scraper = ContentScraper::default();
        let result = scraper.extract(html, "https://example.com");

        assert!(result.is_err());
        matches!(result.unwrap_err(), ScraperError::ExtractionError(_));
    }
}