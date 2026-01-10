//! Media handling for images, video, and audio.
//!
//! This crate provides:
//! - Image decoding (PNG, JPEG, GIF, WebP, SVG)
//! - Video playback (placeholder, would need system integration)
//! - Audio playback (placeholder, would need system integration)
//! - Media session API

pub mod image_decoder;
pub mod video;
pub mod audio;
pub mod media_element;
pub mod media_session;
pub mod canvas;

pub use image_decoder::{ImageDecoder, DecodedImage, ImageFormat};
pub use video::{VideoPlayer, VideoState};
pub use audio::{AudioPlayer, AudioState};
pub use media_element::{MediaElement, MediaReadyState, MediaNetworkState};
pub use media_session::{MediaSession, MediaSessionAction, MediaMetadata};
pub use canvas::{Canvas, CanvasContext, CanvasContext2D};
