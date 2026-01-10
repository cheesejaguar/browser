//! Browser engine - coordinates all browser subsystems.

use std::sync::Arc;
use parking_lot::RwLock;
use url::Url;

use crate::config::BrowserConfig;
use crate::page::Page;

/// The main browser engine.
pub struct BrowserEngine {
    /// Browser configuration.
    config: BrowserConfig,
    /// Open pages.
    pages: RwLock<Vec<Arc<Page>>>,
    /// Active page index.
    active_page: RwLock<Option<usize>>,
    /// Running state.
    running: RwLock<bool>,
}

impl BrowserEngine {
    /// Create a new browser engine.
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            config,
            pages: RwLock::new(Vec::new()),
            active_page: RwLock::new(None),
            running: RwLock::new(false),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(BrowserConfig::default())
    }

    /// Start the browser engine.
    pub fn start(&self) {
        *self.running.write() = true;
        tracing::info!("Browser engine started");
    }

    /// Stop the browser engine.
    pub fn stop(&self) {
        *self.running.write() = false;
        tracing::info!("Browser engine stopped");
    }

    /// Check if running.
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Open a new page.
    pub fn new_page(&self) -> Arc<Page> {
        let page = Arc::new(Page::new(self.config.clone()));
        let mut pages = self.pages.write();
        pages.push(page.clone());
        *self.active_page.write() = Some(pages.len() - 1);
        page
    }

    /// Open a URL in a new page.
    pub async fn open_url(&self, url: &str) -> anyhow::Result<Arc<Page>> {
        let page = self.new_page();
        page.navigate(url).await?;
        Ok(page)
    }

    /// Get the active page.
    pub fn active_page(&self) -> Option<Arc<Page>> {
        let active = *self.active_page.read();
        active.and_then(|idx| self.pages.read().get(idx).cloned())
    }

    /// Set the active page.
    pub fn set_active_page(&self, index: usize) {
        let pages = self.pages.read();
        if index < pages.len() {
            *self.active_page.write() = Some(index);
        }
    }

    /// Get all pages.
    pub fn pages(&self) -> Vec<Arc<Page>> {
        self.pages.read().clone()
    }

    /// Close a page.
    pub fn close_page(&self, index: usize) {
        let mut pages = self.pages.write();
        if index < pages.len() {
            pages.remove(index);

            // Update active page index
            let mut active = self.active_page.write();
            if let Some(current) = *active {
                if current >= pages.len() {
                    *active = if pages.is_empty() {
                        None
                    } else {
                        Some(pages.len() - 1)
                    };
                } else if current > index {
                    *active = Some(current - 1);
                }
            }
        }
    }

    /// Get configuration.
    pub fn config(&self) -> &BrowserConfig {
        &self.config
    }

    /// Get page count.
    pub fn page_count(&self) -> usize {
        self.pages.read().len()
    }
}

impl Default for BrowserEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Browser engine builder.
pub struct BrowserEngineBuilder {
    config: BrowserConfig,
}

impl BrowserEngineBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: BrowserConfig::default(),
        }
    }

    /// Set JavaScript enabled.
    pub fn javascript_enabled(mut self, enabled: bool) -> Self {
        self.config.javascript_enabled = enabled;
        self
    }

    /// Set images enabled.
    pub fn images_enabled(mut self, enabled: bool) -> Self {
        self.config.images_enabled = enabled;
        self
    }

    /// Set user agent.
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.config.user_agent = user_agent.to_string();
        self
    }

    /// Set viewport size.
    pub fn viewport_size(mut self, width: u32, height: u32) -> Self {
        self.config.viewport_width = width;
        self.config.viewport_height = height;
        self
    }

    /// Build the browser engine.
    pub fn build(self) -> BrowserEngine {
        BrowserEngine::new(self.config)
    }
}

impl Default for BrowserEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = BrowserEngine::with_defaults();
        assert!(!engine.is_running());
        assert_eq!(engine.page_count(), 0);
    }

    #[test]
    fn test_engine_start_stop() {
        let engine = BrowserEngine::with_defaults();

        engine.start();
        assert!(engine.is_running());

        engine.stop();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_page_management() {
        let engine = BrowserEngine::with_defaults();

        let _page1 = engine.new_page();
        let _page2 = engine.new_page();

        assert_eq!(engine.page_count(), 2);
        assert!(engine.active_page().is_some());

        engine.close_page(0);
        assert_eq!(engine.page_count(), 1);
    }
}
