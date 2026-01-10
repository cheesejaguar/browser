//! Mixed content blocking.

use url::Url;

/// Mixed content blocker.
#[derive(Clone, Debug)]
pub struct MixedContentBlocker {
    /// Whether strict blocking is enabled.
    strict_mode: bool,
    /// Whether to upgrade insecure requests.
    upgrade_insecure: bool,
}

impl MixedContentBlocker {
    /// Create a new mixed content blocker.
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            upgrade_insecure: false,
        }
    }

    /// Enable strict mode (block all mixed content).
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Enable upgrading insecure requests.
    pub fn set_upgrade_insecure(&mut self, upgrade: bool) {
        self.upgrade_insecure = upgrade;
    }

    /// Check if a resource should be blocked.
    pub fn should_block(&self, page_url: &Url, resource_url: &Url, content_type: MixedContentType) -> MixedContentCheck {
        // Only applies to secure pages
        if page_url.scheme() != "https" {
            return MixedContentCheck::Allowed;
        }

        // Secure resources are always allowed
        if is_secure_scheme(resource_url.scheme()) {
            return MixedContentCheck::Allowed;
        }

        // Check if we should upgrade
        if self.upgrade_insecure && resource_url.scheme() == "http" {
            return MixedContentCheck::Upgrade;
        }

        // In strict mode, block everything
        if self.strict_mode {
            return MixedContentCheck::Blocked(content_type);
        }

        // Apply default blocking rules
        match content_type {
            // Active content is always blocked
            MixedContentType::Script
            | MixedContentType::Stylesheet
            | MixedContentType::Iframe
            | MixedContentType::Object
            | MixedContentType::Fetch
            | MixedContentType::Worker => MixedContentCheck::Blocked(content_type),

            // Passive content may be allowed with warning
            MixedContentType::Image
            | MixedContentType::Audio
            | MixedContentType::Video => MixedContentCheck::Warn(content_type),

            // Other content is blocked
            MixedContentType::Font
            | MixedContentType::Other => MixedContentCheck::Blocked(content_type),
        }
    }

    /// Upgrade an insecure URL to HTTPS if possible.
    pub fn upgrade_url(&self, url: &Url) -> Option<Url> {
        if url.scheme() == "http" {
            let mut upgraded = url.clone();
            let _ = upgraded.set_scheme("https");
            Some(upgraded)
        } else {
            None
        }
    }

    /// Check if strict mode is enabled.
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// Check if upgrade insecure is enabled.
    pub fn is_upgrade_insecure(&self) -> bool {
        self.upgrade_insecure
    }
}

impl Default for MixedContentBlocker {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of mixed content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixedContentType {
    /// Script (active).
    Script,
    /// Stylesheet (active).
    Stylesheet,
    /// Iframe (active).
    Iframe,
    /// Object/embed (active).
    Object,
    /// Fetch/XHR (active).
    Fetch,
    /// Worker (active).
    Worker,
    /// Image (passive).
    Image,
    /// Audio (passive).
    Audio,
    /// Video (passive).
    Video,
    /// Font.
    Font,
    /// Other resources.
    Other,
}

impl MixedContentType {
    /// Check if this is active content.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            MixedContentType::Script
                | MixedContentType::Stylesheet
                | MixedContentType::Iframe
                | MixedContentType::Object
                | MixedContentType::Fetch
                | MixedContentType::Worker
        )
    }

    /// Check if this is passive content.
    pub fn is_passive(&self) -> bool {
        matches!(
            self,
            MixedContentType::Image
                | MixedContentType::Audio
                | MixedContentType::Video
        )
    }

    /// Get the content type from a resource type string.
    pub fn from_resource_type(resource_type: &str) -> Self {
        match resource_type.to_lowercase().as_str() {
            "script" => MixedContentType::Script,
            "stylesheet" | "style" => MixedContentType::Stylesheet,
            "iframe" | "frame" => MixedContentType::Iframe,
            "object" | "embed" => MixedContentType::Object,
            "fetch" | "xhr" | "xmlhttprequest" => MixedContentType::Fetch,
            "worker" | "serviceworker" | "sharedworker" => MixedContentType::Worker,
            "image" | "img" => MixedContentType::Image,
            "audio" => MixedContentType::Audio,
            "video" => MixedContentType::Video,
            "font" => MixedContentType::Font,
            _ => MixedContentType::Other,
        }
    }
}

/// Result of mixed content check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixedContentCheck {
    /// Content is allowed.
    Allowed,
    /// Content should be upgraded to HTTPS.
    Upgrade,
    /// Content is allowed but with a warning.
    Warn(MixedContentType),
    /// Content is blocked.
    Blocked(MixedContentType),
}

impl MixedContentCheck {
    /// Check if the content is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, MixedContentCheck::Allowed | MixedContentCheck::Warn(_))
    }

    /// Check if the content should be upgraded.
    pub fn should_upgrade(&self) -> bool {
        matches!(self, MixedContentCheck::Upgrade)
    }

    /// Check if the content is blocked.
    pub fn is_blocked(&self) -> bool {
        matches!(self, MixedContentCheck::Blocked(_))
    }
}

/// Check if a scheme is considered secure.
fn is_secure_scheme(scheme: &str) -> bool {
    matches!(scheme, "https" | "wss" | "data" | "blob")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_content_active_blocked() {
        let blocker = MixedContentBlocker::new();
        let page_url = Url::parse("https://example.com").unwrap();
        let resource_url = Url::parse("http://cdn.example.com/script.js").unwrap();

        let result = blocker.should_block(&page_url, &resource_url, MixedContentType::Script);
        assert!(result.is_blocked());
    }

    #[test]
    fn test_mixed_content_passive_warned() {
        let blocker = MixedContentBlocker::new();
        let page_url = Url::parse("https://example.com").unwrap();
        let resource_url = Url::parse("http://cdn.example.com/image.png").unwrap();

        let result = blocker.should_block(&page_url, &resource_url, MixedContentType::Image);
        assert!(matches!(result, MixedContentCheck::Warn(_)));
    }

    #[test]
    fn test_mixed_content_upgrade() {
        let mut blocker = MixedContentBlocker::new();
        blocker.set_upgrade_insecure(true);

        let page_url = Url::parse("https://example.com").unwrap();
        let resource_url = Url::parse("http://cdn.example.com/script.js").unwrap();

        let result = blocker.should_block(&page_url, &resource_url, MixedContentType::Script);
        assert!(result.should_upgrade());
    }

    #[test]
    fn test_mixed_content_http_page() {
        let blocker = MixedContentBlocker::new();
        let page_url = Url::parse("http://example.com").unwrap();
        let resource_url = Url::parse("http://cdn.example.com/script.js").unwrap();

        let result = blocker.should_block(&page_url, &resource_url, MixedContentType::Script);
        assert!(result.is_allowed());
    }

    #[test]
    fn test_mixed_content_secure_resource() {
        let blocker = MixedContentBlocker::new();
        let page_url = Url::parse("https://example.com").unwrap();
        let resource_url = Url::parse("https://cdn.example.com/script.js").unwrap();

        let result = blocker.should_block(&page_url, &resource_url, MixedContentType::Script);
        assert!(result.is_allowed());
    }
}
