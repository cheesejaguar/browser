//! Browser configuration.

use crate::user_agent;

/// Browser configuration.
#[derive(Clone, Debug)]
pub struct BrowserConfig {
    /// User agent string.
    pub user_agent: String,
    /// Whether JavaScript is enabled.
    pub javascript_enabled: bool,
    /// Whether images are enabled.
    pub images_enabled: bool,
    /// Whether CSS is enabled.
    pub css_enabled: bool,
    /// Viewport width.
    pub viewport_width: u32,
    /// Viewport height.
    pub viewport_height: u32,
    /// Device pixel ratio.
    pub device_pixel_ratio: f64,
    /// Accept language header.
    pub accept_language: String,
    /// Maximum connections per host.
    pub max_connections_per_host: usize,
    /// Connection timeout in seconds.
    pub connection_timeout: u64,
    /// Whether cookies are enabled.
    pub cookies_enabled: bool,
    /// Whether local storage is enabled.
    pub local_storage_enabled: bool,
    /// Cache size in bytes.
    pub cache_size: usize,
    /// Whether GPU acceleration is enabled.
    pub gpu_acceleration: bool,
    /// Whether hardware video decoding is enabled.
    pub hardware_video_decode: bool,
    /// Whether to block mixed content.
    pub block_mixed_content: bool,
    /// Whether to enforce CSP.
    pub enforce_csp: bool,
    /// Default font family.
    pub default_font: String,
    /// Default font size.
    pub default_font_size: u32,
    /// Minimum font size.
    pub minimum_font_size: u32,
    /// Whether dark mode is preferred.
    pub prefer_dark_mode: bool,
    /// Whether reduced motion is preferred.
    pub prefer_reduced_motion: bool,
}

impl BrowserConfig {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a headless configuration.
    pub fn headless() -> Self {
        Self {
            gpu_acceleration: false,
            viewport_width: 1920,
            viewport_height: 1080,
            ..Self::default()
        }
    }

    /// Create a mobile configuration.
    pub fn mobile() -> Self {
        Self {
            viewport_width: 375,
            viewport_height: 812,
            device_pixel_ratio: 3.0,
            user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".to_string(),
            ..Self::default()
        }
    }

    /// Set viewport size.
    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    /// Set device pixel ratio.
    pub fn with_device_pixel_ratio(mut self, ratio: f64) -> Self {
        self.device_pixel_ratio = ratio;
        self
    }

    /// Set JavaScript enabled.
    pub fn with_javascript(mut self, enabled: bool) -> Self {
        self.javascript_enabled = enabled;
        self
    }

    /// Set images enabled.
    pub fn with_images(mut self, enabled: bool) -> Self {
        self.images_enabled = enabled;
        self
    }

    /// Set user agent.
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            user_agent: user_agent(),
            javascript_enabled: true,
            images_enabled: true,
            css_enabled: true,
            viewport_width: 1280,
            viewport_height: 720,
            device_pixel_ratio: 1.0,
            accept_language: "en-US,en;q=0.9".to_string(),
            max_connections_per_host: 6,
            connection_timeout: 30,
            cookies_enabled: true,
            local_storage_enabled: true,
            cache_size: 100 * 1024 * 1024, // 100MB
            gpu_acceleration: true,
            hardware_video_decode: true,
            block_mixed_content: true,
            enforce_csp: true,
            default_font: "system-ui".to_string(),
            default_font_size: 16,
            minimum_font_size: 9,
            prefer_dark_mode: false,
            prefer_reduced_motion: false,
        }
    }
}

/// Content settings.
#[derive(Clone, Debug)]
pub struct ContentSettings {
    /// JavaScript permissions per origin.
    pub javascript: PermissionSetting,
    /// Cookie permissions per origin.
    pub cookies: PermissionSetting,
    /// Image permissions per origin.
    pub images: PermissionSetting,
    /// Notification permissions per origin.
    pub notifications: PermissionSetting,
    /// Geolocation permissions per origin.
    pub geolocation: PermissionSetting,
    /// Camera permissions per origin.
    pub camera: PermissionSetting,
    /// Microphone permissions per origin.
    pub microphone: PermissionSetting,
}

impl Default for ContentSettings {
    fn default() -> Self {
        Self {
            javascript: PermissionSetting::Allow,
            cookies: PermissionSetting::Allow,
            images: PermissionSetting::Allow,
            notifications: PermissionSetting::Ask,
            geolocation: PermissionSetting::Ask,
            camera: PermissionSetting::Ask,
            microphone: PermissionSetting::Ask,
        }
    }
}

/// Permission setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionSetting {
    /// Always allow.
    Allow,
    /// Always block.
    Block,
    /// Ask the user.
    Ask,
}

/// Privacy settings.
#[derive(Clone, Debug)]
pub struct PrivacySettings {
    /// Whether Do Not Track is enabled.
    pub do_not_track: bool,
    /// Whether third-party cookies are blocked.
    pub block_third_party_cookies: bool,
    /// Whether tracking protection is enabled.
    pub tracking_protection: bool,
    /// Whether fingerprinting protection is enabled.
    pub fingerprinting_protection: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            do_not_track: false,
            block_third_party_cookies: false,
            tracking_protection: false,
            fingerprinting_protection: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BrowserConfig::default();
        assert!(config.javascript_enabled);
        assert!(config.images_enabled);
        assert_eq!(config.viewport_width, 1280);
    }

    #[test]
    fn test_mobile_config() {
        let config = BrowserConfig::mobile();
        assert_eq!(config.viewport_width, 375);
        assert_eq!(config.device_pixel_ratio, 3.0);
    }

    #[test]
    fn test_headless_config() {
        let config = BrowserConfig::headless();
        assert!(!config.gpu_acceleration);
    }

    #[test]
    fn test_config_builder() {
        let config = BrowserConfig::new()
            .with_viewport(1920, 1080)
            .with_javascript(false);

        assert_eq!(config.viewport_width, 1920);
        assert!(!config.javascript_enabled);
    }
}
