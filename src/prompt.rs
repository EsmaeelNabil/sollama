use crate::types::ScrapedContent;

/// The `PromptBuilder` struct is responsible for constructing prompts from scraped content.
/// It allows adding content and building a formatted prompt string.
pub struct PromptBuilder {
    /// The query or question to be included in the prompt.
    query: String,
    /// The list of scraped content to be included in the prompt.
    contents: Vec<ScrapedContent>,
}

impl PromptBuilder {
    /// Creates a new `PromptBuilder` with the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query or question to be included in the prompt.
    ///
    /// # Returns
    ///
    /// A new instance of `PromptBuilder`.
    pub fn new(query: String) -> Self {
        Self {
            query,
            contents: Vec::new(),
        }
    }

    /// Adds the given contents to the `PromptBuilder`.
    ///
    /// # Arguments
    ///
    /// * `contents` - A vector of `ScrapedContent` to be included in the prompt.
    ///
    /// # Returns
    ///
    /// The updated `PromptBuilder` instance.
    pub fn with_contents(mut self, contents: Vec<ScrapedContent>) -> Self {
        self.contents = contents;
        self
    }

    /// Builds the prompt string by formatting the query and contents.
    ///
    /// # Returns
    ///
    /// A formatted prompt string.
    pub fn build(&self) -> String {
        let formatted_contents = self.contents
            .iter()
            .map(|c| {
                Self::clean_text(
                    &format!(
                        "Source: {}\nTimestamp: {}\nContent:\n{}\n---\n",
                        c.url, c.timestamp, c.content
                    )
                )
            })
            .collect::<String>();

        format!(
            "{} {}", self.query,
            formatted_contents
        )
    }

    /// Cleans the given text by removing blank lines and normalizing whitespace.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to be cleaned.
    ///
    /// # Returns
    ///
    /// The cleaned text.
    fn clean_text(text: &str) -> String {
        text.lines()
            .filter(|line| !line.trim().is_empty())       // Remove blank lines
            .map(|line| {
                line.split_whitespace()                   // Split by whitespace
                    .collect::<Vec<&str>>()
                    .join(" ")                            // Join with single space
            })
            .collect::<Vec<String>>()
            .join("\n")                                   // Join lines with newline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    /// Tests the `PromptBuilder` functionality.
    #[test]
    fn test_prompt_builder() {
        let content = ScrapedContent {
            url: "https://example.com".to_string(),
            content: "Test content".to_string(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };

        let prompt = PromptBuilder::new("What is Rust?".to_string())
            .with_contents(vec![content])
            .build();

        assert!(prompt.contains("What is Rust?"));
        assert!(prompt.contains("https://example.com"));
        assert!(prompt.contains("Test content"));
    }
}