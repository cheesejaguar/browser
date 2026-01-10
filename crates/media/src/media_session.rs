//! Media Session API implementation.

use parking_lot::RwLock;
use std::collections::HashMap;

/// Media Session API.
#[derive(Debug)]
pub struct MediaSession {
    /// Current metadata.
    metadata: RwLock<Option<MediaMetadata>>,
    /// Playback state.
    playback_state: RwLock<MediaSessionPlaybackState>,
    /// Action handlers.
    action_handlers: RwLock<HashMap<MediaSessionAction, bool>>,
    /// Position state.
    position_state: RwLock<Option<MediaPositionState>>,
}

impl MediaSession {
    /// Create a new media session.
    pub fn new() -> Self {
        Self {
            metadata: RwLock::new(None),
            playback_state: RwLock::new(MediaSessionPlaybackState::None),
            action_handlers: RwLock::new(HashMap::new()),
            position_state: RwLock::new(None),
        }
    }

    /// Get metadata.
    pub fn metadata(&self) -> Option<MediaMetadata> {
        self.metadata.read().clone()
    }

    /// Set metadata.
    pub fn set_metadata(&self, metadata: Option<MediaMetadata>) {
        *self.metadata.write() = metadata;
    }

    /// Get playback state.
    pub fn playback_state(&self) -> MediaSessionPlaybackState {
        *self.playback_state.read()
    }

    /// Set playback state.
    pub fn set_playback_state(&self, state: MediaSessionPlaybackState) {
        *self.playback_state.write() = state;
    }

    /// Set action handler.
    pub fn set_action_handler(&self, action: MediaSessionAction, has_handler: bool) {
        self.action_handlers.write().insert(action, has_handler);
    }

    /// Check if action handler exists.
    pub fn has_action_handler(&self, action: &MediaSessionAction) -> bool {
        self.action_handlers.read().get(action).copied().unwrap_or(false)
    }

    /// Set position state.
    pub fn set_position_state(&self, state: Option<MediaPositionState>) {
        *self.position_state.write() = state;
    }

    /// Get position state.
    pub fn position_state(&self) -> Option<MediaPositionState> {
        self.position_state.read().clone()
    }

    /// Get supported actions.
    pub fn supported_actions(&self) -> Vec<MediaSessionAction> {
        self.action_handlers
            .read()
            .iter()
            .filter(|(_, &has)| has)
            .map(|(action, _)| *action)
            .collect()
    }
}

impl Default for MediaSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Media metadata.
#[derive(Clone, Debug)]
pub struct MediaMetadata {
    /// Title.
    pub title: String,
    /// Artist.
    pub artist: String,
    /// Album.
    pub album: String,
    /// Artwork.
    pub artwork: Vec<MediaImage>,
}

impl MediaMetadata {
    /// Create new metadata.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            artist: String::new(),
            album: String::new(),
            artwork: Vec::new(),
        }
    }

    /// Set artist.
    pub fn with_artist(mut self, artist: &str) -> Self {
        self.artist = artist.to_string();
        self
    }

    /// Set album.
    pub fn with_album(mut self, album: &str) -> Self {
        self.album = album.to_string();
        self
    }

    /// Add artwork.
    pub fn with_artwork(mut self, artwork: MediaImage) -> Self {
        self.artwork.push(artwork);
        self
    }
}

/// Media image for artwork.
#[derive(Clone, Debug)]
pub struct MediaImage {
    /// Image source URL.
    pub src: String,
    /// Image sizes (e.g., "96x96").
    pub sizes: String,
    /// Image type (e.g., "image/png").
    pub type_: String,
}

impl MediaImage {
    /// Create a new media image.
    pub fn new(src: &str) -> Self {
        Self {
            src: src.to_string(),
            sizes: String::new(),
            type_: String::new(),
        }
    }

    /// Set sizes.
    pub fn with_sizes(mut self, sizes: &str) -> Self {
        self.sizes = sizes.to_string();
        self
    }

    /// Set type.
    pub fn with_type(mut self, type_: &str) -> Self {
        self.type_ = type_.to_string();
        self
    }
}

/// Media session playback state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSessionPlaybackState {
    /// No playback state.
    None,
    /// Paused.
    Paused,
    /// Playing.
    Playing,
}

/// Media session action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MediaSessionAction {
    /// Play action.
    Play,
    /// Pause action.
    Pause,
    /// Seek backward action.
    SeekBackward,
    /// Seek forward action.
    SeekForward,
    /// Previous track action.
    PreviousTrack,
    /// Next track action.
    NextTrack,
    /// Skip ad action.
    SkipAd,
    /// Stop action.
    Stop,
    /// Seek to action.
    SeekTo,
    /// Toggle microphone action.
    ToggleMicrophone,
    /// Toggle camera action.
    ToggleCamera,
    /// Hang up action.
    HangUp,
}

impl MediaSessionAction {
    /// Get action name.
    pub fn name(&self) -> &'static str {
        match self {
            MediaSessionAction::Play => "play",
            MediaSessionAction::Pause => "pause",
            MediaSessionAction::SeekBackward => "seekbackward",
            MediaSessionAction::SeekForward => "seekforward",
            MediaSessionAction::PreviousTrack => "previoustrack",
            MediaSessionAction::NextTrack => "nexttrack",
            MediaSessionAction::SkipAd => "skipad",
            MediaSessionAction::Stop => "stop",
            MediaSessionAction::SeekTo => "seekto",
            MediaSessionAction::ToggleMicrophone => "togglemicrophone",
            MediaSessionAction::ToggleCamera => "togglecamera",
            MediaSessionAction::HangUp => "hangup",
        }
    }

    /// Parse action from name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "play" => Some(MediaSessionAction::Play),
            "pause" => Some(MediaSessionAction::Pause),
            "seekbackward" => Some(MediaSessionAction::SeekBackward),
            "seekforward" => Some(MediaSessionAction::SeekForward),
            "previoustrack" => Some(MediaSessionAction::PreviousTrack),
            "nexttrack" => Some(MediaSessionAction::NextTrack),
            "skipad" => Some(MediaSessionAction::SkipAd),
            "stop" => Some(MediaSessionAction::Stop),
            "seekto" => Some(MediaSessionAction::SeekTo),
            "togglemicrophone" => Some(MediaSessionAction::ToggleMicrophone),
            "togglecamera" => Some(MediaSessionAction::ToggleCamera),
            "hangup" => Some(MediaSessionAction::HangUp),
            _ => None,
        }
    }
}

/// Media position state.
#[derive(Clone, Debug)]
pub struct MediaPositionState {
    /// Duration in seconds.
    pub duration: f64,
    /// Playback rate.
    pub playback_rate: f64,
    /// Position in seconds.
    pub position: f64,
}

impl MediaPositionState {
    /// Create new position state.
    pub fn new(duration: f64, position: f64) -> Self {
        Self {
            duration,
            playback_rate: 1.0,
            position,
        }
    }

    /// Set playback rate.
    pub fn with_playback_rate(mut self, rate: f64) -> Self {
        self.playback_rate = rate;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_session() {
        let session = MediaSession::new();
        assert_eq!(session.playback_state(), MediaSessionPlaybackState::None);
        assert!(session.metadata().is_none());

        let metadata = MediaMetadata::new("Song Title")
            .with_artist("Artist Name")
            .with_album("Album Name");

        session.set_metadata(Some(metadata));
        assert!(session.metadata().is_some());
        assert_eq!(session.metadata().unwrap().title, "Song Title");
    }

    #[test]
    fn test_action_handlers() {
        let session = MediaSession::new();

        session.set_action_handler(MediaSessionAction::Play, true);
        session.set_action_handler(MediaSessionAction::Pause, true);

        assert!(session.has_action_handler(&MediaSessionAction::Play));
        assert!(session.has_action_handler(&MediaSessionAction::Pause));
        assert!(!session.has_action_handler(&MediaSessionAction::NextTrack));
    }
}
