//! Navigator API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::sync::Arc;
use parking_lot::RwLock;

/// Navigator API implementation.
#[derive(Clone, Debug)]
pub struct Navigator {
    /// User agent string.
    user_agent: String,
    /// App name.
    app_name: String,
    /// App version.
    app_version: String,
    /// Platform.
    platform: String,
    /// Language.
    language: String,
    /// Languages.
    languages: Vec<String>,
    /// Online status.
    online: bool,
    /// Cookie enabled.
    cookie_enabled: bool,
    /// Do not track.
    do_not_track: Option<String>,
    /// Max touch points.
    max_touch_points: u32,
    /// Hardware concurrency.
    hardware_concurrency: u32,
    /// Device memory (GB).
    device_memory: f64,
}

impl Navigator {
    /// Create a new Navigator with default values.
    pub fn new() -> Self {
        Self {
            user_agent: format!(
                "Mozilla/5.0 (X11; Linux x86_64) RustBrowser/1.0 (KHTML, like Gecko) Chrome/120.0.0.0"
            ),
            app_name: "Netscape".to_string(), // Standard value
            app_version: "5.0 (X11; Linux x86_64) RustBrowser/1.0".to_string(),
            platform: std::env::consts::OS.to_string(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            online: true,
            cookie_enabled: true,
            do_not_track: None,
            max_touch_points: 0,
            hardware_concurrency: num_cpus::get() as u32,
            device_memory: 8.0,
        }
    }

    /// Create with custom user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set the platform.
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = platform.into();
        self
    }

    /// Set the language.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set online status.
    pub fn set_online(&mut self, online: bool) {
        self.online = online;
    }

    /// Get the user agent.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Get the platform.
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Get the language.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get the languages.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Check if online.
    pub fn online(&self) -> bool {
        self.online
    }

    /// Check if cookies are enabled.
    pub fn cookie_enabled(&self) -> bool {
        self.cookie_enabled
    }

    /// Register the Navigator API on the global object.
    pub fn register(navigator: Arc<RwLock<Navigator>>, context: &mut Context) {
        let nav = navigator.read();

        let navigator_obj = ObjectInitializer::new(context)
            // Properties
            .property(js_string!("userAgent"), js_string!(nav.user_agent.clone()), Attribute::READONLY)
            .property(js_string!("appName"), js_string!(nav.app_name.clone()), Attribute::READONLY)
            .property(js_string!("appVersion"), js_string!(nav.app_version.clone()), Attribute::READONLY)
            .property(js_string!("platform"), js_string!(nav.platform.clone()), Attribute::READONLY)
            .property(js_string!("language"), js_string!(nav.language.clone()), Attribute::READONLY)
            .property(js_string!("onLine"), nav.online, Attribute::READONLY)
            .property(js_string!("cookieEnabled"), nav.cookie_enabled, Attribute::READONLY)
            .property(js_string!("maxTouchPoints"), nav.max_touch_points as i32, Attribute::READONLY)
            .property(js_string!("hardwareConcurrency"), nav.hardware_concurrency as i32, Attribute::READONLY)
            .property(js_string!("deviceMemory"), nav.device_memory, Attribute::READONLY)
            // Methods
            .function(NativeFunction::from_fn_ptr(navigator_send_beacon), js_string!("sendBeacon"), 2)
            .function(NativeFunction::from_fn_ptr(navigator_vibrate), js_string!("vibrate"), 1)
            .function(NativeFunction::from_fn_ptr(navigator_share), js_string!("share"), 1)
            .function(NativeFunction::from_fn_ptr(navigator_can_share), js_string!("canShare"), 1)
            .function(NativeFunction::from_fn_ptr(navigator_register_protocol_handler), js_string!("registerProtocolHandler"), 3)
            .build();

        drop(nav);

        context
            .register_global_property(js_string!("navigator"), navigator_obj, Attribute::all())
            .expect("Failed to register navigator");
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::new()
    }
}

// Native function implementations
fn navigator_send_beacon(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _url = args.get_or_undefined(0).to_string(context)?;
    let _data = args.get(1);

    // In a real implementation, this would send a beacon request
    Ok(JsValue::from(true))
}

fn navigator_vibrate(_: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // Vibration is typically not supported on desktop
    Ok(JsValue::from(false))
}

fn navigator_share(_: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a rejected promise (not implemented)
    use boa_engine::object::builtins::JsPromise;
    let (promise, resolvers) = JsPromise::new_pending(context);
    // Would reject with NotAllowedError
    Ok(promise.into())
}

fn navigator_can_share(_: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(false))
}

fn navigator_register_protocol_handler(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _scheme = args.get_or_undefined(0).to_string(context)?;
    let _url = args.get_or_undefined(1).to_string(context)?;
    // Third argument (title) is deprecated
    Ok(JsValue::undefined())
}

/// Geolocation API (subset of Navigator).
#[derive(Clone, Debug, Default)]
pub struct Geolocation {
    /// Mock position for testing.
    mock_position: Option<GeolocationPosition>,
}

impl Geolocation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a mock position for testing.
    pub fn set_mock_position(&mut self, position: GeolocationPosition) {
        self.mock_position = Some(position);
    }
}

/// Geolocation position.
#[derive(Clone, Debug)]
pub struct GeolocationPosition {
    /// Latitude.
    pub latitude: f64,
    /// Longitude.
    pub longitude: f64,
    /// Altitude (optional).
    pub altitude: Option<f64>,
    /// Accuracy in meters.
    pub accuracy: f64,
    /// Altitude accuracy (optional).
    pub altitude_accuracy: Option<f64>,
    /// Heading (optional).
    pub heading: Option<f64>,
    /// Speed (optional).
    pub speed: Option<f64>,
    /// Timestamp.
    pub timestamp: u64,
}

/// Network information API.
#[derive(Clone, Debug)]
pub struct NetworkInformation {
    /// Connection type.
    pub connection_type: ConnectionType,
    /// Effective connection type.
    pub effective_type: EffectiveConnectionType,
    /// Downlink speed in Mbps.
    pub downlink: f64,
    /// Round-trip time in ms.
    pub rtt: u32,
    /// Save data mode.
    pub save_data: bool,
}

impl Default for NetworkInformation {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Unknown,
            effective_type: EffectiveConnectionType::FourG,
            downlink: 10.0,
            rtt: 50,
            save_data: false,
        }
    }
}

/// Connection type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionType {
    Bluetooth,
    Cellular,
    Ethernet,
    None,
    Wifi,
    Wimax,
    Other,
    Unknown,
}

/// Effective connection type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectiveConnectionType {
    Slow2G,
    TwoG,
    ThreeG,
    FourG,
}

/// Media devices API.
pub struct MediaDevices;

impl MediaDevices {
    /// Register the mediaDevices API.
    pub fn register(context: &mut Context) {
        let media_devices = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(media_devices_enumerate_devices), js_string!("enumerateDevices"), 0)
            .function(NativeFunction::from_fn_ptr(media_devices_get_user_media), js_string!("getUserMedia"), 1)
            .function(NativeFunction::from_fn_ptr(media_devices_get_display_media), js_string!("getDisplayMedia"), 1)
            .build();

        // Add to navigator
        // In a real implementation, this would be a property of navigator
    }
}

fn media_devices_enumerate_devices(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn media_devices_get_user_media(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

fn media_devices_get_display_media(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use boa_engine::object::builtins::JsPromise;
    let (promise, _) = JsPromise::new_pending(context);
    Ok(promise.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigator_creation() {
        let navigator = Navigator::new();
        assert!(!navigator.user_agent().is_empty());
        assert!(navigator.online());
        assert!(navigator.cookie_enabled());
    }

    #[test]
    fn test_navigator_customization() {
        let navigator = Navigator::new()
            .with_user_agent("CustomBrowser/1.0")
            .with_language("es-ES");

        assert_eq!(navigator.user_agent(), "CustomBrowser/1.0");
        assert_eq!(navigator.language(), "es-ES");
    }

    #[test]
    fn test_network_information() {
        let info = NetworkInformation::default();
        assert_eq!(info.effective_type, EffectiveConnectionType::FourG);
        assert!(info.downlink > 0.0);
    }
}
