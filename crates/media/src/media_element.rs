//! HTMLMediaElement base implementation.

use std::time::Duration;
use parking_lot::RwLock;

/// Media element (base for audio and video).
#[derive(Debug)]
pub struct MediaElement {
    /// Source URL.
    src: RwLock<String>,
    /// Current source URL (after processing).
    current_src: RwLock<String>,
    /// Cross-origin attribute.
    cross_origin: RwLock<Option<CrossOrigin>>,
    /// Network state.
    network_state: RwLock<MediaNetworkState>,
    /// Ready state.
    ready_state: RwLock<MediaReadyState>,
    /// Seeking flag.
    seeking: RwLock<bool>,
    /// Current time.
    current_time: RwLock<Duration>,
    /// Duration.
    duration: RwLock<Option<Duration>>,
    /// Paused flag.
    paused: RwLock<bool>,
    /// Default playback rate.
    default_playback_rate: RwLock<f64>,
    /// Playback rate.
    playback_rate: RwLock<f64>,
    /// Played time ranges.
    played: RwLock<Vec<TimeRange>>,
    /// Seekable time ranges.
    seekable: RwLock<Vec<TimeRange>>,
    /// Ended flag.
    ended: RwLock<bool>,
    /// Autoplay flag.
    autoplay: RwLock<bool>,
    /// Loop flag.
    loop_: RwLock<bool>,
    /// Controls flag.
    controls: RwLock<bool>,
    /// Volume.
    volume: RwLock<f64>,
    /// Muted flag.
    muted: RwLock<bool>,
    /// Default muted flag.
    default_muted: RwLock<bool>,
    /// Preload hint.
    preload: RwLock<Preload>,
    /// Error.
    error: RwLock<Option<MediaError>>,
}

impl MediaElement {
    /// Create a new media element.
    pub fn new() -> Self {
        Self {
            src: RwLock::new(String::new()),
            current_src: RwLock::new(String::new()),
            cross_origin: RwLock::new(None),
            network_state: RwLock::new(MediaNetworkState::Empty),
            ready_state: RwLock::new(MediaReadyState::HaveNothing),
            seeking: RwLock::new(false),
            current_time: RwLock::new(Duration::ZERO),
            duration: RwLock::new(None),
            paused: RwLock::new(true),
            default_playback_rate: RwLock::new(1.0),
            playback_rate: RwLock::new(1.0),
            played: RwLock::new(Vec::new()),
            seekable: RwLock::new(Vec::new()),
            ended: RwLock::new(false),
            autoplay: RwLock::new(false),
            loop_: RwLock::new(false),
            controls: RwLock::new(false),
            volume: RwLock::new(1.0),
            muted: RwLock::new(false),
            default_muted: RwLock::new(false),
            preload: RwLock::new(Preload::Auto),
            error: RwLock::new(None),
        }
    }

    // Source attributes

    /// Get src attribute.
    pub fn src(&self) -> String {
        self.src.read().clone()
    }

    /// Set src attribute.
    pub fn set_src(&self, src: &str) {
        *self.src.write() = src.to_string();
        self.load();
    }

    /// Get current src.
    pub fn current_src(&self) -> String {
        self.current_src.read().clone()
    }

    /// Get cross-origin attribute.
    pub fn cross_origin(&self) -> Option<CrossOrigin> {
        *self.cross_origin.read()
    }

    /// Set cross-origin attribute.
    pub fn set_cross_origin(&self, value: Option<CrossOrigin>) {
        *self.cross_origin.write() = value;
    }

    // Network state

    /// Get network state.
    pub fn network_state(&self) -> MediaNetworkState {
        *self.network_state.read()
    }

    /// Get preload hint.
    pub fn preload(&self) -> Preload {
        *self.preload.read()
    }

    /// Set preload hint.
    pub fn set_preload(&self, preload: Preload) {
        *self.preload.write() = preload;
    }

    // Ready state

    /// Get ready state.
    pub fn ready_state(&self) -> MediaReadyState {
        *self.ready_state.read()
    }

    /// Check if seeking.
    pub fn seeking(&self) -> bool {
        *self.seeking.read()
    }

    // Playback state

    /// Get current time.
    pub fn current_time(&self) -> Duration {
        *self.current_time.read()
    }

    /// Set current time (seek).
    pub fn set_current_time(&self, time: Duration) {
        *self.seeking.write() = true;
        *self.current_time.write() = time;
        *self.seeking.write() = false;
    }

    /// Get duration.
    pub fn duration(&self) -> Option<Duration> {
        *self.duration.read()
    }

    /// Check if paused.
    pub fn paused(&self) -> bool {
        *self.paused.read()
    }

    /// Get default playback rate.
    pub fn default_playback_rate(&self) -> f64 {
        *self.default_playback_rate.read()
    }

    /// Set default playback rate.
    pub fn set_default_playback_rate(&self, rate: f64) {
        *self.default_playback_rate.write() = rate;
    }

    /// Get playback rate.
    pub fn playback_rate(&self) -> f64 {
        *self.playback_rate.read()
    }

    /// Set playback rate.
    pub fn set_playback_rate(&self, rate: f64) {
        *self.playback_rate.write() = rate.clamp(0.0625, 16.0);
    }

    /// Get played ranges.
    pub fn played(&self) -> Vec<TimeRange> {
        self.played.read().clone()
    }

    /// Get seekable ranges.
    pub fn seekable(&self) -> Vec<TimeRange> {
        self.seekable.read().clone()
    }

    /// Check if ended.
    pub fn ended(&self) -> bool {
        *self.ended.read()
    }

    /// Get autoplay.
    pub fn autoplay(&self) -> bool {
        *self.autoplay.read()
    }

    /// Set autoplay.
    pub fn set_autoplay(&self, autoplay: bool) {
        *self.autoplay.write() = autoplay;
    }

    /// Get loop.
    pub fn loop_(&self) -> bool {
        *self.loop_.read()
    }

    /// Set loop.
    pub fn set_loop(&self, loop_: bool) {
        *self.loop_.write() = loop_;
    }

    // Controls

    /// Get controls.
    pub fn controls(&self) -> bool {
        *self.controls.read()
    }

    /// Set controls.
    pub fn set_controls(&self, controls: bool) {
        *self.controls.write() = controls;
    }

    /// Get volume.
    pub fn volume(&self) -> f64 {
        *self.volume.read()
    }

    /// Set volume.
    pub fn set_volume(&self, volume: f64) -> Result<(), MediaError> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(MediaError::InvalidValue);
        }
        *self.volume.write() = volume;
        Ok(())
    }

    /// Check if muted.
    pub fn muted(&self) -> bool {
        *self.muted.read()
    }

    /// Set muted.
    pub fn set_muted(&self, muted: bool) {
        *self.muted.write() = muted;
    }

    /// Get default muted.
    pub fn default_muted(&self) -> bool {
        *self.default_muted.read()
    }

    /// Set default muted.
    pub fn set_default_muted(&self, muted: bool) {
        *self.default_muted.write() = muted;
    }

    // Methods

    /// Load the media resource.
    pub fn load(&self) {
        *self.network_state.write() = MediaNetworkState::Loading;
        *self.ready_state.write() = MediaReadyState::HaveNothing;
        *self.error.write() = None;
        *self.current_src.write() = self.src.read().clone();
    }

    /// Play the media.
    pub fn play(&self) -> Result<(), MediaError> {
        if *self.ready_state.read() == MediaReadyState::HaveNothing {
            return Err(MediaError::NotReady);
        }
        *self.paused.write() = false;
        *self.ended.write() = false;
        Ok(())
    }

    /// Pause the media.
    pub fn pause(&self) {
        *self.paused.write() = true;
    }

    /// Check if media can play a type.
    pub fn can_play_type(&self, mime_type: &str) -> CanPlayType {
        // Simplified - in reality this would check codec support
        match mime_type {
            "video/mp4" | "video/webm" | "video/ogg" => CanPlayType::Probably,
            "audio/mpeg" | "audio/ogg" | "audio/wav" | "audio/webm" => CanPlayType::Probably,
            "video/quicktime" | "audio/aac" => CanPlayType::Maybe,
            _ => CanPlayType::Empty,
        }
    }

    /// Get error.
    pub fn error(&self) -> Option<MediaError> {
        self.error.read().clone()
    }

    /// Set ready state.
    pub fn set_ready_state(&self, state: MediaReadyState) {
        *self.ready_state.write() = state;
    }

    /// Set network state.
    pub fn set_network_state(&self, state: MediaNetworkState) {
        *self.network_state.write() = state;
    }

    /// Set duration.
    pub fn set_duration(&self, duration: Duration) {
        *self.duration.write() = Some(duration);
    }

    /// Set error.
    pub fn set_error(&self, error: MediaError) {
        *self.error.write() = Some(error);
    }
}

impl Default for MediaElement {
    fn default() -> Self {
        Self::new()
    }
}

/// Time range.
#[derive(Clone, Debug)]
pub struct TimeRange {
    pub start: Duration,
    pub end: Duration,
}

/// Media ready state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaReadyState {
    /// No information about media.
    HaveNothing = 0,
    /// Metadata available.
    HaveMetadata = 1,
    /// Current frame available.
    HaveCurrentData = 2,
    /// Future data available.
    HaveFutureData = 3,
    /// Enough data for playback.
    HaveEnoughData = 4,
}

/// Media network state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaNetworkState {
    /// Not initialized.
    Empty = 0,
    /// Idle (no activity).
    Idle = 1,
    /// Loading.
    Loading = 2,
    /// No source found.
    NoSource = 3,
}

/// Cross-origin attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossOrigin {
    /// Anonymous.
    Anonymous,
    /// Use credentials.
    UseCredentials,
}

/// Preload hint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preload {
    /// Don't preload.
    None,
    /// Preload metadata only.
    Metadata,
    /// Preload entire resource.
    Auto,
}

/// Can play type result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanPlayType {
    /// Cannot play.
    Empty,
    /// Might be able to play.
    Maybe,
    /// Probably can play.
    Probably,
}

impl CanPlayType {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            CanPlayType::Empty => "",
            CanPlayType::Maybe => "maybe",
            CanPlayType::Probably => "probably",
        }
    }
}

/// Media error.
#[derive(Clone, Debug, thiserror::Error)]
pub enum MediaError {
    #[error("Media aborted")]
    Aborted,

    #[error("Network error")]
    Network,

    #[error("Decode error")]
    Decode,

    #[error("Source not supported")]
    SrcNotSupported,

    #[error("Not ready")]
    NotReady,

    #[error("Invalid value")]
    InvalidValue,
}

impl MediaError {
    /// Get error code.
    pub fn code(&self) -> u16 {
        match self {
            MediaError::Aborted => 1,
            MediaError::Network => 2,
            MediaError::Decode => 3,
            MediaError::SrcNotSupported => 4,
            MediaError::NotReady => 0,
            MediaError::InvalidValue => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_element() {
        let element = MediaElement::new();
        assert_eq!(element.ready_state(), MediaReadyState::HaveNothing);
        assert!(element.paused());

        element.set_src("https://example.com/video.mp4");
        assert_eq!(element.network_state(), MediaNetworkState::Loading);
    }

    #[test]
    fn test_can_play_type() {
        let element = MediaElement::new();
        assert_eq!(element.can_play_type("video/mp4"), CanPlayType::Probably);
        assert_eq!(element.can_play_type("video/unknown"), CanPlayType::Empty);
    }

    #[test]
    fn test_volume() {
        let element = MediaElement::new();
        assert!(element.set_volume(0.5).is_ok());
        assert_eq!(element.volume(), 0.5);

        assert!(element.set_volume(1.5).is_err());
        assert!(element.set_volume(-0.1).is_err());
    }
}
