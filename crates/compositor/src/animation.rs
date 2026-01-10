//! Animation support for compositor.

use common::geometry::Transform;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Animation identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationId(pub u64);

/// Animation state.
#[derive(Clone, Debug)]
pub struct Animation {
    /// Animation ID.
    pub id: AnimationId,
    /// Start time.
    pub start_time: Instant,
    /// Duration.
    pub duration: Duration,
    /// Delay before starting.
    pub delay: Duration,
    /// Number of iterations (0 = infinite).
    pub iterations: u32,
    /// Current iteration.
    pub current_iteration: u32,
    /// Animation direction.
    pub direction: AnimationDirection,
    /// Fill mode.
    pub fill_mode: FillMode,
    /// Play state.
    pub play_state: PlayState,
    /// Easing function.
    pub easing: Easing,
    /// Keyframes.
    pub keyframes: Vec<Keyframe>,
}

impl Animation {
    pub fn new(id: AnimationId, duration: Duration) -> Self {
        Self {
            id,
            start_time: Instant::now(),
            duration,
            delay: Duration::ZERO,
            iterations: 1,
            current_iteration: 0,
            direction: AnimationDirection::Normal,
            fill_mode: FillMode::None,
            play_state: PlayState::Running,
            easing: Easing::Linear,
            keyframes: Vec::new(),
        }
    }

    /// Check if animation is finished.
    pub fn is_finished(&self) -> bool {
        if self.iterations == 0 {
            return false; // Infinite
        }
        self.current_iteration >= self.iterations
    }

    /// Get the current progress (0.0 - 1.0).
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed();

        if elapsed < self.delay {
            return match self.fill_mode {
                FillMode::Backwards | FillMode::Both => 0.0,
                _ => return 0.0,
            };
        }

        let elapsed_after_delay = elapsed - self.delay;
        let iteration_progress = if self.duration.is_zero() {
            1.0
        } else {
            (elapsed_after_delay.as_secs_f32() / self.duration.as_secs_f32()) % 1.0
        };

        // Apply direction
        let progress = match self.direction {
            AnimationDirection::Normal => iteration_progress,
            AnimationDirection::Reverse => 1.0 - iteration_progress,
            AnimationDirection::Alternate => {
                if self.current_iteration % 2 == 0 {
                    iteration_progress
                } else {
                    1.0 - iteration_progress
                }
            }
            AnimationDirection::AlternateReverse => {
                if self.current_iteration % 2 == 0 {
                    1.0 - iteration_progress
                } else {
                    iteration_progress
                }
            }
        };

        // Apply easing
        self.easing.apply(progress)
    }

    /// Update the animation.
    pub fn update(&mut self) {
        if self.play_state != PlayState::Running {
            return;
        }

        let elapsed = self.start_time.elapsed();
        if elapsed < self.delay {
            return;
        }

        let elapsed_after_delay = elapsed - self.delay;
        let total_iterations = if self.duration.is_zero() {
            self.iterations
        } else {
            (elapsed_after_delay.as_secs_f32() / self.duration.as_secs_f32()).floor() as u32
        };

        self.current_iteration = total_iterations.min(self.iterations.saturating_sub(1));
    }

    /// Interpolate between keyframes at the current progress.
    pub fn interpolate(&self) -> AnimatedValues {
        let progress = self.progress();

        if self.keyframes.is_empty() {
            return AnimatedValues::default();
        }

        // Find surrounding keyframes
        let mut prev = &self.keyframes[0];
        let mut next = &self.keyframes[0];

        for keyframe in &self.keyframes {
            if keyframe.offset <= progress {
                prev = keyframe;
            }
            if keyframe.offset >= progress {
                next = keyframe;
                break;
            }
        }

        // Interpolate
        let local_progress = if (next.offset - prev.offset).abs() < f32::EPSILON {
            1.0
        } else {
            (progress - prev.offset) / (next.offset - prev.offset)
        };

        AnimatedValues {
            opacity: Some(lerp(
                prev.values.opacity.unwrap_or(1.0),
                next.values.opacity.unwrap_or(1.0),
                local_progress,
            )),
            transform: prev.values.transform.as_ref().map(|prev_t| {
                let next_t = next.values.transform.as_ref().unwrap_or(prev_t);
                prev_t.interpolate(next_t, local_progress)
            }),
        }
    }
}

/// Animation direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

/// Fill mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

/// Play state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayState {
    Running,
    Paused,
}

/// Easing function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
    Steps(u32, StepPosition),
}

/// Step position for step easing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    Start,
    End,
}

impl Easing {
    /// Apply the easing function to a progress value.
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Easing::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            Easing::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            Easing::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(*x1, *y1, *x2, *y2, t),
            Easing::Steps(steps, position) => {
                let step = (t * *steps as f32).floor() as f32;
                match position {
                    StepPosition::Start => (step + 1.0) / *steps as f32,
                    StepPosition::End => step / *steps as f32,
                }
            }
        }
    }
}

/// A keyframe in an animation.
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// Offset in the animation (0.0 - 1.0).
    pub offset: f32,
    /// Easing for this segment.
    pub easing: Option<Easing>,
    /// Animated values.
    pub values: AnimatedValues,
}

/// Animated property values.
#[derive(Clone, Debug, Default)]
pub struct AnimatedValues {
    pub opacity: Option<f32>,
    pub transform: Option<Transform>,
}

/// Animation controller.
pub struct AnimationController {
    /// Active animations.
    animations: HashMap<AnimationId, Animation>,
    /// Animation counter for IDs.
    id_counter: u64,
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            id_counter: 0,
        }
    }

    /// Create a new animation.
    pub fn create_animation(&mut self, duration: Duration) -> AnimationId {
        self.id_counter += 1;
        let id = AnimationId(self.id_counter);
        let animation = Animation::new(id, duration);
        self.animations.insert(id, animation);
        id
    }

    /// Get an animation.
    pub fn get(&self, id: AnimationId) -> Option<&Animation> {
        self.animations.get(&id)
    }

    /// Get a mutable animation.
    pub fn get_mut(&mut self, id: AnimationId) -> Option<&mut Animation> {
        self.animations.get_mut(&id)
    }

    /// Remove an animation.
    pub fn remove(&mut self, id: AnimationId) {
        self.animations.remove(&id);
    }

    /// Update all animations.
    pub fn update(&mut self) {
        let mut finished = Vec::new();

        for (id, animation) in &mut self.animations {
            animation.update();
            if animation.is_finished() {
                finished.push(*id);
            }
        }

        // Remove finished animations
        for id in finished {
            self.animations.remove(&id);
        }
    }

    /// Get all active animations.
    pub fn active_animations(&self) -> impl Iterator<Item = &Animation> {
        self.animations.values()
    }

    /// Check if any animations are running.
    pub fn has_active_animations(&self) -> bool {
        self.animations.values().any(|a| a.play_state == PlayState::Running)
    }
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

/// Linear interpolation.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Cubic bezier easing.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    // Simplified cubic bezier - for production would use Newton's method
    let t2 = t * t;
    let t3 = t2 * t;

    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;

    let cy = 3.0 * y1;
    let by = 3.0 * (y2 - y1) - cy;
    let ay = 1.0 - cy - by;

    // Solve for t given x (simplified)
    let x = ax * t3 + bx * t2 + cx * t;

    // Calculate y
    ay * t3 + by * t2 + cy * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_creation() {
        let mut controller = AnimationController::new();
        let id = controller.create_animation(Duration::from_secs(1));

        let animation = controller.get(id).unwrap();
        assert_eq!(animation.duration, Duration::from_secs(1));
        assert!(!animation.is_finished());
    }

    #[test]
    fn test_easing() {
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 0.001);

        // Ease should be faster in the middle
        let ease_mid = Easing::Ease.apply(0.5);
        assert!(ease_mid > 0.4 && ease_mid < 0.9);
    }

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 100.0, 0.5) - 50.0).abs() < 0.001);
        assert!((lerp(0.0, 100.0, 0.0) - 0.0).abs() < 0.001);
        assert!((lerp(0.0, 100.0, 1.0) - 100.0).abs() < 0.001);
    }
}
