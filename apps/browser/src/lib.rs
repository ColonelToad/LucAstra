//! LucAstra Lightweight Browser
//!
//! A minimal HTTP client and HTML renderer for text-based web browsing.
//! Features: HTTP GET, basic HTML parsing, tabs, history, bookmarks.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;
use regex::Regex;

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type BrowserResult<T> = Result<T, BrowserError>;

/// Parsed HTML content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlContent {
    pub title: String,
    pub text: String,
    pub links: Vec<Link>,
    pub images: Vec<String>,
}

/// A hyperlink found in HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub text: String,
    pub href: String,
}

/// HTTP client for fetching pages
pub struct HttpClient {
    user_agent: String,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            user_agent: "LucAstra-Browser/1.0".to_string(),
        }
    }

    /// Fetch HTML from a URL (blocking)
    pub fn get(&self, url: &str) -> BrowserResult<String> {
        // Validate URL format
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(BrowserError::InvalidUrl(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        // Use reqwest synchronously
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", self.user_agent.clone())
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| BrowserError::NetworkError(e.to_string()))?;

        response
            .text()
            .map_err(|e| BrowserError::NetworkError(e.to_string()))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple HTML parser and renderer
pub struct HtmlParser;

impl HtmlParser {
    /// Parse HTML content into text, links, images
    pub fn parse(html: &str) -> HtmlContent {
        let title = Self::extract_title(html);
        let text = Self::extract_text(html);
        let links = Self::extract_links(html);
        let images = Self::extract_images(html);

        HtmlContent {
            title,
            text,
            links,
            images,
        }
    }

    /// Extract page title from <title> tag
    fn extract_title(html: &str) -> String {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html.find("</title>") {
                if start < end {
                    let title = &html[start + 7..end];
                    return title.trim().to_string();
                }
            }
        }
        "Untitled".to_string()
    }

    /// Extract plain text from HTML (strip tags)
    fn extract_text(html: &str) -> String {
        let mut text = String::new();
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;

        for line in html.lines() {
            // Skip script and style tags
            if line.contains("<script") {
                in_script = true;
            }
            if line.contains("</script>") {
                in_script = false;
            }
            if line.contains("<style") {
                in_style = true;
            }
            if line.contains("</style>") {
                in_style = false;
            }

            if in_script || in_style {
                continue;
            }

            for ch in line.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => {
                        in_tag = false;
                        if !text.ends_with('\n') {
                            text.push(' ');
                        }
                    }
                    _ if !in_tag => text.push(ch),
                    _ => {}
                }
            }
            text.push('\n');
        }

        // Decode HTML entities and normalize whitespace
        let text = Self::decode_entities(&text);
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        lines.join("\n")
    }

    /// Extract links from <a> tags
    fn extract_links(html: &str) -> Vec<Link> {
        let mut links = Vec::new();
        let href_pattern = Regex::new(r#"<a\s+[^>]*href=["']([^"']+)["'][^>]*>([^<]*)</a>"#).unwrap();

        for cap in href_pattern.captures_iter(html) {
            if let (Some(href_m), Some(text_m)) = (cap.get(1), cap.get(2)) {
                links.push(Link {
                    text: text_m.as_str().trim().to_string(),
                    href: href_m.as_str().to_string(),
                });
            }
        }

        links
    }

    /// Extract image URLs from <img> tags
    fn extract_images(html: &str) -> Vec<String> {
        let mut images = Vec::new();
        let img_pattern = Regex::new(r#"<img\s+[^>]*src=["']([^"']+)["'][^>]*>"#).unwrap();

        for cap in img_pattern.captures_iter(html) {
            if let Some(src_m) = cap.get(1) {
                images.push(src_m.as_str().to_string());
            }
        }

        images
    }

    /// Decode basic HTML entities
    fn decode_entities(text: &str) -> String {
        text.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
    }
}

/// Browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub url: String,
    pub content: Option<HtmlContent>,
    pub history: VecDeque<String>,
}

impl Tab {
    pub fn new(url: String) -> Self {
        Self {
            url,
            content: None,
            history: VecDeque::new(),
        }
    }

    /// Load URL in this tab
    pub fn load(&mut self, url: String, client: &HttpClient) -> BrowserResult<()> {
        let html = client.get(&url)?;
        let content = HtmlParser::parse(&html);

        self.history.push_front(self.url.clone());
        self.url = url;
        self.content = Some(content);

        Ok(())
    }

    /// Go back in history
    pub fn back(&mut self, client: &HttpClient) -> BrowserResult<()> {
        if let Some(prev_url) = self.history.pop_front() {
            self.load(prev_url, client)?;
        }
        Ok(())
    }
}

/// Browser state with tabs and bookmarks
#[derive(Debug, Clone)]
pub struct Browser {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub bookmarks: Vec<String>,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab::new("about:blank".to_string())],
            active_tab: 0,
            bookmarks: Vec::new(),
        }
    }

    /// Get active tab (mutable)
    pub fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Get active tab (immutable)
    pub fn current_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    /// Open new tab
    pub fn new_tab(&mut self, url: String) {
        self.tabs.push(Tab::new(url));
        self.active_tab = self.tabs.len() - 1;
    }

    /// Close active tab
    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Switch to tab by index
    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    /// Add URL to bookmarks
    pub fn bookmark(&mut self) {
        if let Some(tab) = self.current_tab() {
            if !self.bookmarks.contains(&tab.url) {
                self.bookmarks.push(tab.url.clone());
            }
        }
    }

    /// Navigate to URL in current tab
    pub fn navigate(&mut self, url: String, client: &HttpClient) -> BrowserResult<()> {
        if let Some(tab) = self.current_tab_mut() {
            tab.load(url, client)?;
        }
        Ok(())
    }

    /// Go back in current tab
    pub fn back(&mut self, client: &HttpClient) -> BrowserResult<()> {
        if let Some(tab) = self.current_tab_mut() {
            tab.back(client)?;
        }
        Ok(())
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser_extract_title() {
        let html = "<html><head><title>My Page</title></head></html>";
        let content = HtmlParser::parse(html);
        assert_eq!(content.title, "My Page");
    }

    #[test]
    fn test_html_parser_extract_text() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let content = HtmlParser::parse(html);
        assert!(content.text.contains("Hello"));
        assert!(content.text.contains("World"));
    }

    #[test]
    fn test_html_parser_extract_links() {
        let html = r#"<a href="https://example.com">Example</a><a href="/page">Page</a>"#;
        let content = HtmlParser::parse(html);
        assert_eq!(content.links.len(), 2);
        assert_eq!(content.links[0].text, "Example");
        assert_eq!(content.links[0].href, "https://example.com");
    }

    #[test]
    fn test_html_parser_extract_images() {
        let html = r#"<img src="image1.png"><img src="image2.jpg">"#;
        let content = HtmlParser::parse(html);
        assert_eq!(content.images.len(), 2);
        assert_eq!(content.images[0], "image1.png");
    }

    #[test]
    fn test_browser_tabs() {
        let mut browser = Browser::new();
        assert_eq!(browser.tabs.len(), 1);

        browser.new_tab("https://example.com".to_string());
        assert_eq!(browser.tabs.len(), 2);
        assert_eq!(browser.active_tab, 1);

        browser.switch_tab(0);
        assert_eq!(browser.active_tab, 0);

        browser.close_tab();
        assert_eq!(browser.tabs.len(), 1);
    }

    #[test]
    fn test_browser_bookmarks() {
        let mut browser = Browser::new();
        let tab = Tab::new("https://example.com".to_string());
        browser.tabs[0] = tab;

        browser.bookmark();
        assert!(browser.bookmarks.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_http_client_url_validation() {
        let client = HttpClient::new();
        let result = client.get("invalid-url");
        assert!(result.is_err());
    }
}
