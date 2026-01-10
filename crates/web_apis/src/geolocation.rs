//! Geolocation API stub.

/// Geolocation API (stub implementation).
pub struct Geolocation;

/// Position coordinates.
#[derive(Clone, Debug)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

/// Position.
#[derive(Clone, Debug)]
pub struct Position {
    pub coords: Coordinates,
    pub timestamp: u64,
}

/// Position options.
#[derive(Clone, Debug, Default)]
pub struct PositionOptions {
    pub enable_high_accuracy: bool,
    pub timeout: Option<u32>,
    pub maximum_age: Option<u32>,
}

/// Position error.
#[derive(Clone, Debug)]
pub struct PositionError {
    pub code: PositionErrorCode,
    pub message: String,
}

/// Position error code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionErrorCode {
    PermissionDenied = 1,
    PositionUnavailable = 2,
    Timeout = 3,
}
