//! Permissions API implementation.

use std::collections::HashMap;
use parking_lot::RwLock;
use crate::origin::Origin;

/// Permission types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Geolocation access.
    Geolocation,
    /// Camera access.
    Camera,
    /// Microphone access.
    Microphone,
    /// Notifications.
    Notifications,
    /// Push notifications.
    Push,
    /// Background sync.
    BackgroundSync,
    /// Clipboard read.
    ClipboardRead,
    /// Clipboard write.
    ClipboardWrite,
    /// Persistent storage.
    PersistentStorage,
    /// Ambient light sensor.
    AmbientLightSensor,
    /// Accelerometer.
    Accelerometer,
    /// Gyroscope.
    Gyroscope,
    /// Magnetometer.
    Magnetometer,
    /// Screen wake lock.
    ScreenWakeLock,
    /// MIDI.
    Midi,
    /// Bluetooth.
    Bluetooth,
    /// USB.
    Usb,
    /// NFC.
    Nfc,
    /// Display capture.
    DisplayCapture,
    /// Window placement.
    WindowPlacement,
}

impl Permission {
    /// Get permission name.
    pub fn name(&self) -> &'static str {
        match self {
            Permission::Geolocation => "geolocation",
            Permission::Camera => "camera",
            Permission::Microphone => "microphone",
            Permission::Notifications => "notifications",
            Permission::Push => "push",
            Permission::BackgroundSync => "background-sync",
            Permission::ClipboardRead => "clipboard-read",
            Permission::ClipboardWrite => "clipboard-write",
            Permission::PersistentStorage => "persistent-storage",
            Permission::AmbientLightSensor => "ambient-light-sensor",
            Permission::Accelerometer => "accelerometer",
            Permission::Gyroscope => "gyroscope",
            Permission::Magnetometer => "magnetometer",
            Permission::ScreenWakeLock => "screen-wake-lock",
            Permission::Midi => "midi",
            Permission::Bluetooth => "bluetooth",
            Permission::Usb => "usb",
            Permission::Nfc => "nfc",
            Permission::DisplayCapture => "display-capture",
            Permission::WindowPlacement => "window-placement",
        }
    }

    /// Parse permission from name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "geolocation" => Some(Permission::Geolocation),
            "camera" => Some(Permission::Camera),
            "microphone" => Some(Permission::Microphone),
            "notifications" => Some(Permission::Notifications),
            "push" => Some(Permission::Push),
            "background-sync" => Some(Permission::BackgroundSync),
            "clipboard-read" => Some(Permission::ClipboardRead),
            "clipboard-write" => Some(Permission::ClipboardWrite),
            "persistent-storage" => Some(Permission::PersistentStorage),
            "ambient-light-sensor" => Some(Permission::AmbientLightSensor),
            "accelerometer" => Some(Permission::Accelerometer),
            "gyroscope" => Some(Permission::Gyroscope),
            "magnetometer" => Some(Permission::Magnetometer),
            "screen-wake-lock" => Some(Permission::ScreenWakeLock),
            "midi" => Some(Permission::Midi),
            "bluetooth" => Some(Permission::Bluetooth),
            "usb" => Some(Permission::Usb),
            "nfc" => Some(Permission::Nfc),
            "display-capture" => Some(Permission::DisplayCapture),
            "window-placement" => Some(Permission::WindowPlacement),
            _ => None,
        }
    }

    /// Check if this permission requires secure context.
    pub fn requires_secure_context(&self) -> bool {
        matches!(
            self,
            Permission::Geolocation
                | Permission::Camera
                | Permission::Microphone
                | Permission::Notifications
                | Permission::Push
                | Permission::BackgroundSync
                | Permission::ClipboardRead
                | Permission::ClipboardWrite
                | Permission::Bluetooth
                | Permission::Usb
                | Permission::Nfc
                | Permission::DisplayCapture
        )
    }

    /// Check if this permission requires user activation.
    pub fn requires_user_activation(&self) -> bool {
        matches!(
            self,
            Permission::Camera
                | Permission::Microphone
                | Permission::Notifications
                | Permission::Bluetooth
                | Permission::Usb
                | Permission::Nfc
                | Permission::DisplayCapture
        )
    }
}

/// Permission state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionState {
    /// Permission granted.
    Granted,
    /// Permission denied.
    Denied,
    /// Permission needs to be requested.
    Prompt,
}

impl PermissionState {
    /// Check if permission is granted.
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionState::Granted)
    }

    /// Check if permission is denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, PermissionState::Denied)
    }
}

/// Permissions manager.
#[derive(Debug, Default)]
pub struct PermissionsManager {
    /// Permission states per origin.
    states: RwLock<HashMap<String, HashMap<Permission, PermissionState>>>,
    /// Default permission states.
    defaults: HashMap<Permission, PermissionState>,
}

impl PermissionsManager {
    /// Create a new permissions manager.
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        // Most permissions default to prompt
        for permission in [
            Permission::Geolocation,
            Permission::Camera,
            Permission::Microphone,
            Permission::Notifications,
            Permission::Push,
            Permission::Bluetooth,
            Permission::Usb,
            Permission::Nfc,
            Permission::DisplayCapture,
        ] {
            defaults.insert(permission, PermissionState::Prompt);
        }

        // Some permissions default to granted
        defaults.insert(Permission::ClipboardWrite, PermissionState::Granted);

        Self {
            states: RwLock::new(HashMap::new()),
            defaults,
        }
    }

    /// Query permission state.
    pub fn query(&self, origin: &Origin, permission: Permission) -> PermissionState {
        let states = self.states.read();
        let origin_key = origin.serialize();

        if let Some(origin_states) = states.get(&origin_key) {
            if let Some(state) = origin_states.get(&permission) {
                return *state;
            }
        }

        self.defaults.get(&permission).copied().unwrap_or(PermissionState::Prompt)
    }

    /// Set permission state.
    pub fn set(&self, origin: &Origin, permission: Permission, state: PermissionState) {
        let mut states = self.states.write();
        let origin_key = origin.serialize();

        states
            .entry(origin_key)
            .or_insert_with(HashMap::new)
            .insert(permission, state);
    }

    /// Reset permission state to default.
    pub fn reset(&self, origin: &Origin, permission: Permission) {
        let mut states = self.states.write();
        let origin_key = origin.serialize();

        if let Some(origin_states) = states.get_mut(&origin_key) {
            origin_states.remove(&permission);
        }
    }

    /// Reset all permissions for an origin.
    pub fn reset_all(&self, origin: &Origin) {
        let mut states = self.states.write();
        let origin_key = origin.serialize();
        states.remove(&origin_key);
    }

    /// Get all permission states for an origin.
    pub fn get_all(&self, origin: &Origin) -> HashMap<Permission, PermissionState> {
        let states = self.states.read();
        let origin_key = origin.serialize();

        states.get(&origin_key).cloned().unwrap_or_default()
    }
}

/// Permissions Policy (Feature Policy) implementation.
#[derive(Clone, Debug, Default)]
pub struct PermissionsPolicy {
    /// Policy directives.
    directives: HashMap<String, PolicyDirective>,
}

impl PermissionsPolicy {
    /// Create a new empty policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a Permissions-Policy header.
    pub fn parse(header: &str) -> Self {
        let mut policy = Self::new();

        for directive_str in header.split(',') {
            let directive_str = directive_str.trim();
            if directive_str.is_empty() {
                continue;
            }

            // Parse feature=value
            if let Some(eq_idx) = directive_str.find('=') {
                let feature = directive_str[..eq_idx].trim();
                let value = directive_str[eq_idx + 1..].trim();

                let directive = PolicyDirective::parse(value);
                policy.directives.insert(feature.to_string(), directive);
            }
        }

        policy
    }

    /// Check if a feature is allowed for an origin.
    pub fn is_feature_allowed(&self, feature: &str, origin: &Origin, is_self: bool) -> bool {
        match self.directives.get(feature) {
            Some(directive) => directive.allows(origin, is_self),
            None => true, // Default allow if not specified
        }
    }

    /// Check if a feature is allowed for self.
    pub fn is_feature_allowed_for_self(&self, feature: &str) -> bool {
        match self.directives.get(feature) {
            Some(directive) => directive.allows_self,
            None => true,
        }
    }
}

/// Policy directive.
#[derive(Clone, Debug, Default)]
pub struct PolicyDirective {
    /// Allow self.
    allows_self: bool,
    /// Allow all origins.
    allows_all: bool,
    /// Specific allowed origins.
    allowed_origins: Vec<String>,
}

impl PolicyDirective {
    /// Parse a directive value.
    pub fn parse(value: &str) -> Self {
        let mut directive = Self::default();

        // Remove parentheses if present
        let value = value.trim_matches(|c| c == '(' || c == ')');

        for token in value.split_whitespace() {
            let token = token.trim_matches('"');
            match token {
                "*" => directive.allows_all = true,
                "self" => directive.allows_self = true,
                origin => directive.allowed_origins.push(origin.to_string()),
            }
        }

        directive
    }

    /// Check if an origin is allowed.
    pub fn allows(&self, origin: &Origin, is_self: bool) -> bool {
        if self.allows_all {
            return true;
        }

        if is_self && self.allows_self {
            return true;
        }

        let origin_str = origin.serialize();
        self.allowed_origins.iter().any(|o| o == &origin_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_query() {
        let manager = PermissionsManager::new();
        let origin = Origin::parse("https://example.com").unwrap();

        // Default state
        assert_eq!(manager.query(&origin, Permission::Geolocation), PermissionState::Prompt);

        // Set state
        manager.set(&origin, Permission::Geolocation, PermissionState::Granted);
        assert_eq!(manager.query(&origin, Permission::Geolocation), PermissionState::Granted);

        // Reset
        manager.reset(&origin, Permission::Geolocation);
        assert_eq!(manager.query(&origin, Permission::Geolocation), PermissionState::Prompt);
    }

    #[test]
    fn test_permissions_policy_parse() {
        let policy = PermissionsPolicy::parse("geolocation=(self), camera=*");

        let origin = Origin::parse("https://example.com").unwrap();
        assert!(policy.is_feature_allowed_for_self("geolocation"));
        assert!(policy.is_feature_allowed("camera", &origin, false));
    }

    #[test]
    fn test_permission_requires_secure_context() {
        assert!(Permission::Geolocation.requires_secure_context());
        assert!(Permission::Camera.requires_secure_context());
        assert!(!Permission::PersistentStorage.requires_secure_context());
    }
}
