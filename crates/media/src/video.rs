//! Video playback.

use std::time::Duration;
use parking_lot::RwLock;

/// Video player.
#[derive(Debug)]
pub struct VideoPlayer {
    /// Video source URL.
    source: RwLock<Option<String>>,
    /// Current state.
    state: RwLock<VideoState>,
    /// Video dimensions.
    dimensions: RwLock<(u32, u32)>,
    /// Current playback position.
    position: RwLock<Duration>,
    /// Video duration.
    duration: RwLock<Option<Duration>>,
    /// Playback rate.
    playback_rate: RwLock<f64>,
    /// Volume (0.0 to 1.0).
    volume: RwLock<f64>,
    /// Muted state.
    muted: RwLock<bool>,
    /// Loop playback.
    looping: RwLock<bool>,
    /// Autoplay.
    autoplay: RwLock<bool>,
}

impl VideoPlayer {
    /// Create a new video player.
    pub fn new() -> Self {
        Self {
            source: RwLock::new(None),
            state: RwLock::new(VideoState::Idle),
            dimensions: RwLock::new((0, 0)),
            position: RwLock::new(Duration::ZERO),
            duration: RwLock::new(None),
            playback_rate: RwLock::new(1.0),
            volume: RwLock::new(1.0),
            muted: RwLock::new(false),
            looping: RwLock::new(false),
            autoplay: RwLock::new(false),
        }
    }

    /// Set the video source.
    pub fn set_source(&self, url: &str) {
        *self.source.write() = Some(url.to_string());
        *self.state.write() = VideoState::Loading;
    }

    /// Get the video source.
    pub fn source(&self) -> Option<String> {
        self.source.read().clone()
    }

    /// Get the current state.
    pub fn state(&self) -> VideoState {
        self.state.read().clone()
    }

    /// Play the video.
    pub fn play(&self) -> Result<(), VideoError> {
        let mut state = self.state.write();
        match *state {
            VideoState::Idle => Err(VideoError::NoSource),
            VideoState::Loading => Err(VideoError::NotReady),
            VideoState::Error(_) => Err(VideoError::InErrorState),
            VideoState::Playing => Ok(()), // Already playing
            VideoState::Paused | VideoState::Ready | VideoState::Ended => {
                *state = VideoState::Playing;
                Ok(())
            }
        }
    }

    /// Pause the video.
    pub fn pause(&self) {
        let mut state = self.state.write();
        if *state == VideoState::Playing {
            *state = VideoState::Paused;
        }
    }

    /// Stop the video.
    pub fn stop(&self) {
        *self.state.write() = VideoState::Ready;
        *self.position.write() = Duration::ZERO;
    }

    /// Seek to a position.
    pub fn seek(&self, position: Duration) -> Result<(), VideoError> {
        let duration = self.duration.read();
        if let Some(dur) = *duration {
            if position > dur {
                return Err(VideoError::SeekOutOfRange);
            }
        }
        *self.position.write() = position;
        Ok(())
    }

    /// Get current position.
    pub fn current_time(&self) -> Duration {
        *self.position.read()
    }

    /// Set current position.
    pub fn set_current_time(&self, time: Duration) {
        *self.position.write() = time;
    }

    /// Get duration.
    pub fn duration(&self) -> Option<Duration> {
        *self.duration.read()
    }

    /// Get video dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        *self.dimensions.read()
    }

    /// Set video dimensions.
    pub fn set_dimensions(&self, width: u32, height: u32) {
        *self.dimensions.write() = (width, height);
    }

    /// Get playback rate.
    pub fn playback_rate(&self) -> f64 {
        *self.playback_rate.read()
    }

    /// Set playback rate.
    pub fn set_playback_rate(&self, rate: f64) {
        *self.playback_rate.write() = rate.clamp(0.25, 4.0);
    }

    /// Get volume.
    pub fn volume(&self) -> f64 {
        *self.volume.read()
    }

    /// Set volume.
    pub fn set_volume(&self, volume: f64) {
        *self.volume.write() = volume.clamp(0.0, 1.0);
    }

    /// Check if muted.
    pub fn muted(&self) -> bool {
        *self.muted.read()
    }

    /// Set muted state.
    pub fn set_muted(&self, muted: bool) {
        *self.muted.write() = muted;
    }

    /// Check if looping.
    pub fn looping(&self) -> bool {
        *self.looping.read()
    }

    /// Set loop state.
    pub fn set_looping(&self, looping: bool) {
        *self.looping.write() = looping;
    }

    /// Check if autoplay.
    pub fn autoplay(&self) -> bool {
        *self.autoplay.read()
    }

    /// Set autoplay.
    pub fn set_autoplay(&self, autoplay: bool) {
        *self.autoplay.write() = autoplay;
    }

    /// Check if ended.
    pub fn ended(&self) -> bool {
        self.state.read().clone() == VideoState::Ended
    }

    /// Check if paused.
    pub fn paused(&self) -> bool {
        let state = self.state.read().clone();
        state == VideoState::Paused || state == VideoState::Idle || state == VideoState::Ready
    }

    /// Get buffered ranges (placeholder).
    pub fn buffered(&self) -> Vec<TimeRange> {
        // Would return actual buffered ranges in real implementation
        if let Some(dur) = self.duration.read().clone() {
            vec![TimeRange {
                start: Duration::ZERO,
                end: dur,
            }]
        } else {
            vec![]
        }
    }

    /// Get seekable ranges.
    pub fn seekable(&self) -> Vec<TimeRange> {
        self.buffered()
    }

    /// Update playback state (called each frame).
    pub fn update(&self, delta: Duration) {
        if self.state.read().clone() != VideoState::Playing {
            return;
        }

        let rate = *self.playback_rate.read();
        let mut position = self.position.write();
        *position += Duration::from_secs_f64(delta.as_secs_f64() * rate);

        // Check for end of video
        if let Some(duration) = *self.duration.read() {
            if *position >= duration {
                if *self.looping.read() {
                    *position = Duration::ZERO;
                } else {
                    *position = duration;
                    *self.state.write() = VideoState::Ended;
                }
            }
        }
    }

    /// Mark as ready.
    pub fn set_ready(&self, duration: Duration, width: u32, height: u32) {
        *self.duration.write() = Some(duration);
        *self.dimensions.write() = (width, height);
        *self.state.write() = VideoState::Ready;
    }

    /// Mark as error.
    pub fn set_error(&self, error: String) {
        *self.state.write() = VideoState::Error(error);
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Video playback state.
#[derive(Clone, Debug, PartialEq)]
pub enum VideoState {
    /// No source set.
    Idle,
    /// Loading video.
    Loading,
    /// Ready to play.
    Ready,
    /// Currently playing.
    Playing,
    /// Paused.
    Paused,
    /// Ended.
    Ended,
    /// Error state.
    Error(String),
}

/// Time range.
#[derive(Clone, Debug)]
pub struct TimeRange {
    pub start: Duration,
    pub end: Duration,
}

impl TimeRange {
    /// Get duration of this range.
    pub fn duration(&self) -> Duration {
        self.end.saturating_sub(self.start)
    }
}

/// Video error.
#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("No video source")]
    NoSource,

    #[error("Video not ready")]
    NotReady,

    #[error("Video in error state")]
    InErrorState,

    #[error("Seek position out of range")]
    SeekOutOfRange,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Decode error: {0}")]
    DecodeError(String),
}

/// Video format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4,
    WebM,
    Ogg,
    Avi,
    Mkv,
}

impl VideoFormat {
    /// Detect format from MIME type.
    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "video/mp4" => Some(VideoFormat::Mp4),
            "video/webm" => Some(VideoFormat::WebM),
            "video/ogg" => Some(VideoFormat::Ogg),
            "video/x-msvideo" => Some(VideoFormat::Avi),
            "video/x-matroska" => Some(VideoFormat::Mkv),
            _ => None,
        }
    }

    /// Get MIME type.
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "video/mp4",
            VideoFormat::WebM => "video/webm",
            VideoFormat::Ogg => "video/ogg",
            VideoFormat::Avi => "video/x-msvideo",
            VideoFormat::Mkv => "video/x-matroska",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_player() {
        let player = VideoPlayer::new();
        assert_eq!(player.state(), VideoState::Idle);

        player.set_source("https://example.com/video.mp4");
        assert_eq!(player.state(), VideoState::Loading);

        player.set_ready(Duration::from_secs(60), 1920, 1080);
        assert_eq!(player.state(), VideoState::Ready);
        assert_eq!(player.dimensions(), (1920, 1080));

        player.play().unwrap();
        assert_eq!(player.state(), VideoState::Playing);

        player.pause();
        assert_eq!(player.state(), VideoState::Paused);
    }

    #[test]
    fn test_volume_clamping() {
        let player = VideoPlayer::new();

        player.set_volume(1.5);
        assert_eq!(player.volume(), 1.0);

        player.set_volume(-0.5);
        assert_eq!(player.volume(), 0.0);

        player.set_volume(0.5);
        assert_eq!(player.volume(), 0.5);
    }
}
