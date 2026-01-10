//! Browser page implementation.

use std::sync::Arc;
use parking_lot::RwLock;
use url::Url;

use crate::config::BrowserConfig;
use crate::pipeline::RenderPipeline;

/// A browser page (tab).
pub struct Page {
    /// Page configuration.
    config: BrowserConfig,
    /// Current URL.
    url: RwLock<Option<Url>>,
    /// Page title.
    title: RwLock<String>,
    /// Loading state.
    loading: RwLock<bool>,
    /// Load progress (0.0 to 1.0).
    progress: RwLock<f32>,
    /// Render pipeline.
    pipeline: RwLock<Option<RenderPipeline>>,
    /// Navigation history.
    history: RwLock<NavigationHistory>,
    /// Page content (raw HTML).
    content: RwLock<String>,
    /// Security state.
    security_state: RwLock<SecurityState>,
}

impl Page {
    /// Create a new page.
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            config,
            url: RwLock::new(None),
            title: RwLock::new(String::new()),
            loading: RwLock::new(false),
            progress: RwLock::new(0.0),
            pipeline: RwLock::new(None),
            history: RwLock::new(NavigationHistory::new()),
            content: RwLock::new(String::new()),
            security_state: RwLock::new(SecurityState::Unknown),
        }
    }

    /// Navigate to a URL.
    pub async fn navigate(&self, url: &str) -> anyhow::Result<()> {
        let parsed_url = if url.contains("://") {
            Url::parse(url)?
        } else {
            Url::parse(&format!("https://{}", url))?
        };

        // Start loading
        *self.loading.write() = true;
        *self.progress.write() = 0.0;

        // Update URL
        *self.url.write() = Some(parsed_url.clone());

        // Add to history
        self.history.write().push(parsed_url.clone());

        // Update security state
        *self.security_state.write() = if parsed_url.scheme() == "https" {
            SecurityState::Secure
        } else {
            SecurityState::Insecure
        };

        // Simulate loading progress
        *self.progress.write() = 0.5;

        // Load content (placeholder - would use networking crate)
        tracing::info!("Navigating to: {}", parsed_url);

        // Simulate page load completion
        *self.progress.write() = 1.0;
        *self.loading.write() = false;

        Ok(())
    }

    /// Set page content directly (for testing).
    pub fn set_content(&self, html: &str) {
        *self.content.write() = html.to_string();
    }

    /// Get current URL.
    pub fn url(&self) -> Option<Url> {
        self.url.read().clone()
    }

    /// Get page title.
    pub fn title(&self) -> String {
        self.title.read().clone()
    }

    /// Set page title.
    pub fn set_title(&self, title: &str) {
        *self.title.write() = title.to_string();
    }

    /// Check if loading.
    pub fn is_loading(&self) -> bool {
        *self.loading.read()
    }

    /// Get load progress.
    pub fn progress(&self) -> f32 {
        *self.progress.read()
    }

    /// Get security state.
    pub fn security_state(&self) -> SecurityState {
        *self.security_state.read()
    }

    /// Go back in history.
    pub async fn go_back(&self) -> anyhow::Result<bool> {
        if let Some(url) = self.history.write().back() {
            *self.url.write() = Some(url.clone());
            // Would reload page content
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Go forward in history.
    pub async fn go_forward(&self) -> anyhow::Result<bool> {
        if let Some(url) = self.history.write().forward() {
            *self.url.write() = Some(url.clone());
            // Would reload page content
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reload the page.
    pub async fn reload(&self) -> anyhow::Result<()> {
        if let Some(url) = self.url() {
            self.navigate(url.as_str()).await?;
        }
        Ok(())
    }

    /// Stop loading.
    pub fn stop(&self) {
        *self.loading.write() = false;
    }

    /// Check if can go back.
    pub fn can_go_back(&self) -> bool {
        self.history.read().can_go_back()
    }

    /// Check if can go forward.
    pub fn can_go_forward(&self) -> bool {
        self.history.read().can_go_forward()
    }

    /// Get page content.
    pub fn content(&self) -> String {
        self.content.read().clone()
    }

    /// Execute JavaScript.
    pub fn evaluate_script(&self, _script: &str) -> anyhow::Result<String> {
        // Would use js_engine crate
        Ok(String::new())
    }

    /// Take a screenshot (returns raw RGBA data).
    pub fn screenshot(&self) -> Option<Vec<u8>> {
        // Would use render pipeline
        None
    }

    /// Get configuration.
    pub fn config(&self) -> &BrowserConfig {
        &self.config
    }
}

/// Navigation history.
#[derive(Debug)]
pub struct NavigationHistory {
    /// History entries.
    entries: Vec<Url>,
    /// Current position.
    position: usize,
}

impl NavigationHistory {
    /// Create a new history.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
        }
    }

    /// Push a URL to history.
    pub fn push(&mut self, url: Url) {
        // Remove forward entries if we're not at the end
        if self.position < self.entries.len() {
            self.entries.truncate(self.position);
        }

        self.entries.push(url);
        self.position = self.entries.len();
    }

    /// Go back.
    pub fn back(&mut self) -> Option<&Url> {
        if self.position > 1 {
            self.position -= 1;
            self.entries.get(self.position - 1)
        } else {
            None
        }
    }

    /// Go forward.
    pub fn forward(&mut self) -> Option<&Url> {
        if self.position < self.entries.len() {
            self.position += 1;
            self.entries.get(self.position - 1)
        } else {
            None
        }
    }

    /// Can go back.
    pub fn can_go_back(&self) -> bool {
        self.position > 1
    }

    /// Can go forward.
    pub fn can_go_forward(&self) -> bool {
        self.position < self.entries.len()
    }

    /// Get current entry.
    pub fn current(&self) -> Option<&Url> {
        if self.position > 0 {
            self.entries.get(self.position - 1)
        } else {
            None
        }
    }

    /// Get all entries.
    pub fn entries(&self) -> &[Url] {
        &self.entries
    }

    /// Get position.
    pub fn position(&self) -> usize {
        self.position
    }
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Security state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityState {
    /// Unknown state.
    Unknown,
    /// Secure (HTTPS).
    Secure,
    /// Insecure (HTTP).
    Insecure,
    /// Broken (mixed content, cert issues).
    Broken,
}

impl SecurityState {
    /// Check if secure.
    pub fn is_secure(&self) -> bool {
        matches!(self, SecurityState::Secure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_page_navigation() {
        let page = Page::new(BrowserConfig::default());

        page.navigate("https://example.com").await.unwrap();

        assert!(page.url().is_some());
        assert_eq!(page.url().unwrap().host_str(), Some("example.com"));
        assert!(page.security_state().is_secure());
    }

    #[test]
    fn test_navigation_history() {
        let mut history = NavigationHistory::new();

        history.push(Url::parse("https://example.com").unwrap());
        history.push(Url::parse("https://example.com/page1").unwrap());
        history.push(Url::parse("https://example.com/page2").unwrap());

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.back();
        assert!(history.can_go_forward());
    }
}
