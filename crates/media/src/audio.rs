//! Audio playback.

use std::time::Duration;
use parking_lot::RwLock;

/// Audio player.
#[derive(Debug)]
pub struct AudioPlayer {
    /// Audio source URL.
    source: RwLock<Option<String>>,
    /// Current state.
    state: RwLock<AudioState>,
    /// Current playback position.
    position: RwLock<Duration>,
    /// Audio duration.
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

impl AudioPlayer {
    /// Create a new audio player.
    pub fn new() -> Self {
        Self {
            source: RwLock::new(None),
            state: RwLock::new(AudioState::Idle),
            position: RwLock::new(Duration::ZERO),
            duration: RwLock::new(None),
            playback_rate: RwLock::new(1.0),
            volume: RwLock::new(1.0),
            muted: RwLock::new(false),
            looping: RwLock::new(false),
            autoplay: RwLock::new(false),
        }
    }

    /// Set the audio source.
    pub fn set_source(&self, url: &str) {
        *self.source.write() = Some(url.to_string());
        *self.state.write() = AudioState::Loading;
    }

    /// Get the audio source.
    pub fn source(&self) -> Option<String> {
        self.source.read().clone()
    }

    /// Get the current state.
    pub fn state(&self) -> AudioState {
        self.state.read().clone()
    }

    /// Play the audio.
    pub fn play(&self) -> Result<(), AudioError> {
        let mut state = self.state.write();
        match *state {
            AudioState::Idle => Err(AudioError::NoSource),
            AudioState::Loading => Err(AudioError::NotReady),
            AudioState::Error(_) => Err(AudioError::InErrorState),
            AudioState::Playing => Ok(()), // Already playing
            AudioState::Paused | AudioState::Ready | AudioState::Ended => {
                *state = AudioState::Playing;
                Ok(())
            }
        }
    }

    /// Pause the audio.
    pub fn pause(&self) {
        let mut state = self.state.write();
        if *state == AudioState::Playing {
            *state = AudioState::Paused;
        }
    }

    /// Stop the audio.
    pub fn stop(&self) {
        *self.state.write() = AudioState::Ready;
        *self.position.write() = Duration::ZERO;
    }

    /// Seek to a position.
    pub fn seek(&self, position: Duration) -> Result<(), AudioError> {
        let duration = self.duration.read();
        if let Some(dur) = *duration {
            if position > dur {
                return Err(AudioError::SeekOutOfRange);
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
        self.state.read().clone() == AudioState::Ended
    }

    /// Check if paused.
    pub fn paused(&self) -> bool {
        let state = self.state.read().clone();
        state == AudioState::Paused || state == AudioState::Idle || state == AudioState::Ready
    }

    /// Update playback state (called each frame).
    pub fn update(&self, delta: Duration) {
        if self.state.read().clone() != AudioState::Playing {
            return;
        }

        let rate = *self.playback_rate.read();
        let mut position = self.position.write();
        *position += Duration::from_secs_f64(delta.as_secs_f64() * rate);

        // Check for end of audio
        if let Some(duration) = self.duration.read().clone() {
            if *position >= duration {
                if *self.looping.read() {
                    *position = Duration::ZERO;
                } else {
                    *position = duration;
                    *self.state.write() = AudioState::Ended;
                }
            }
        }
    }

    /// Mark as ready.
    pub fn set_ready(&self, duration: Duration) {
        *self.duration.write() = Some(duration);
        *self.state.write() = AudioState::Ready;
    }

    /// Mark as error.
    pub fn set_error(&self, error: String) {
        *self.state.write() = AudioState::Error(error);
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio playback state.
#[derive(Clone, Debug, PartialEq)]
pub enum AudioState {
    /// No source set.
    Idle,
    /// Loading audio.
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

/// Audio error.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("No audio source")]
    NoSource,

    #[error("Audio not ready")]
    NotReady,

    #[error("Audio in error state")]
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

/// Audio format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Aac,
    Flac,
    WebM,
}

impl AudioFormat {
    /// Detect format from MIME type.
    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "audio/mpeg" | "audio/mp3" => Some(AudioFormat::Mp3),
            "audio/wav" | "audio/x-wav" => Some(AudioFormat::Wav),
            "audio/ogg" => Some(AudioFormat::Ogg),
            "audio/aac" => Some(AudioFormat::Aac),
            "audio/flac" => Some(AudioFormat::Flac),
            "audio/webm" => Some(AudioFormat::WebM),
            _ => None,
        }
    }

    /// Get MIME type.
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Aac => "audio/aac",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::WebM => "audio/webm",
        }
    }
}

/// Web Audio API - AudioContext placeholder.
#[derive(Debug)]
pub struct AudioContext {
    /// Sample rate.
    sample_rate: f32,
    /// Current time.
    current_time: f64,
    /// State.
    state: AudioContextState,
}

impl AudioContext {
    /// Create a new audio context.
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            current_time: 0.0,
            state: AudioContextState::Suspended,
        }
    }

    /// Get sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get current time.
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Get state.
    pub fn state(&self) -> AudioContextState {
        self.state
    }

    /// Resume the context.
    pub fn resume(&mut self) {
        self.state = AudioContextState::Running;
    }

    /// Suspend the context.
    pub fn suspend(&mut self) {
        self.state = AudioContextState::Suspended;
    }

    /// Close the context.
    pub fn close(&mut self) {
        self.state = AudioContextState::Closed;
    }
}

impl Default for AudioContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio context state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioContextState {
    /// Suspended.
    Suspended,
    /// Running.
    Running,
    /// Closed.
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player() {
        let player = AudioPlayer::new();
        assert_eq!(player.state(), AudioState::Idle);

        player.set_source("https://example.com/audio.mp3");
        assert_eq!(player.state(), AudioState::Loading);

        player.set_ready(Duration::from_secs(180));
        assert_eq!(player.state(), AudioState::Ready);

        player.play().unwrap();
        assert_eq!(player.state(), AudioState::Playing);

        player.pause();
        assert_eq!(player.state(), AudioState::Paused);
    }

    #[test]
    fn test_audio_context() {
        let mut ctx = AudioContext::new();
        assert_eq!(ctx.state(), AudioContextState::Suspended);

        ctx.resume();
        assert_eq!(ctx.state(), AudioContextState::Running);

        ctx.close();
        assert_eq!(ctx.state(), AudioContextState::Closed);
    }
}
