//! JavaScript runtime environment.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// JavaScript runtime.
pub struct Runtime {
    /// Pending timers.
    timers: Vec<Timer>,
    /// Timer ID counter.
    timer_counter: u32,
    /// Microtask queue.
    microtasks: VecDeque<Microtask>,
    /// Macrotask queue.
    macrotasks: VecDeque<Macrotask>,
    /// Animation frame callbacks.
    animation_frames: Vec<AnimationFrameCallback>,
    /// Animation frame ID counter.
    animation_frame_counter: u32,
    /// Idle callbacks.
    idle_callbacks: Vec<IdleCallback>,
    /// Idle callback ID counter.
    idle_callback_counter: u32,
    /// Whether the runtime is running.
    running: bool,
}

impl Runtime {
    /// Create a new runtime.
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            timer_counter: 0,
            microtasks: VecDeque::new(),
            macrotasks: VecDeque::new(),
            animation_frames: Vec::new(),
            animation_frame_counter: 0,
            idle_callbacks: Vec::new(),
            idle_callback_counter: 0,
            running: true,
        }
    }

    /// Add a timer.
    pub fn add_timer(&mut self, callback: TimerCallback, delay: Duration, repeat: bool) -> u32 {
        self.timer_counter += 1;
        let id = self.timer_counter;

        let timer = Timer {
            id,
            callback,
            scheduled_at: Instant::now(),
            delay,
            repeat,
            cancelled: false,
        };

        self.timers.push(timer);
        id
    }

    /// Cancel a timer.
    pub fn cancel_timer(&mut self, id: u32) {
        if let Some(timer) = self.timers.iter_mut().find(|t| t.id == id) {
            timer.cancelled = true;
        }
    }

    /// Get timers that are ready to fire.
    pub fn get_ready_timers(&mut self) -> Vec<Timer> {
        let now = Instant::now();
        let mut ready = Vec::new();
        let mut remaining = Vec::new();

        for timer in self.timers.drain(..) {
            if timer.cancelled {
                continue;
            }

            if now.duration_since(timer.scheduled_at) >= timer.delay {
                if timer.repeat {
                    // Reschedule repeating timer
                    remaining.push(Timer {
                        id: timer.id,
                        callback: timer.callback.clone(),
                        scheduled_at: now,
                        delay: timer.delay,
                        repeat: true,
                        cancelled: false,
                    });
                }
                ready.push(timer);
            } else {
                remaining.push(timer);
            }
        }

        self.timers = remaining;
        ready
    }

    /// Queue a microtask.
    pub fn queue_microtask(&mut self, task: Microtask) {
        self.microtasks.push_back(task);
    }

    /// Get the next microtask.
    pub fn next_microtask(&mut self) -> Option<Microtask> {
        self.microtasks.pop_front()
    }

    /// Check if there are pending microtasks.
    pub fn has_microtasks(&self) -> bool {
        !self.microtasks.is_empty()
    }

    /// Queue a macrotask.
    pub fn queue_macrotask(&mut self, task: Macrotask) {
        self.macrotasks.push_back(task);
    }

    /// Get the next macrotask.
    pub fn next_macrotask(&mut self) -> Option<Macrotask> {
        self.macrotasks.pop_front()
    }

    /// Check if there are pending macrotasks.
    pub fn has_macrotasks(&self) -> bool {
        !self.macrotasks.is_empty()
    }

    /// Request animation frame.
    pub fn request_animation_frame(&mut self, callback: AnimationFrameCallbackFn) -> u32 {
        self.animation_frame_counter += 1;
        let id = self.animation_frame_counter;

        self.animation_frames.push(AnimationFrameCallback {
            id,
            callback,
            cancelled: false,
        });

        id
    }

    /// Cancel animation frame.
    pub fn cancel_animation_frame(&mut self, id: u32) {
        if let Some(callback) = self.animation_frames.iter_mut().find(|c| c.id == id) {
            callback.cancelled = true;
        }
    }

    /// Get animation frame callbacks to run.
    pub fn drain_animation_frames(&mut self) -> Vec<AnimationFrameCallback> {
        let callbacks: Vec<_> = self
            .animation_frames
            .drain(..)
            .filter(|c| !c.cancelled)
            .collect();
        callbacks
    }

    /// Request idle callback.
    pub fn request_idle_callback(&mut self, callback: IdleCallbackFn, timeout: Option<Duration>) -> u32 {
        self.idle_callback_counter += 1;
        let id = self.idle_callback_counter;

        self.idle_callbacks.push(IdleCallback {
            id,
            callback,
            timeout,
            requested_at: Instant::now(),
            cancelled: false,
        });

        id
    }

    /// Cancel idle callback.
    pub fn cancel_idle_callback(&mut self, id: u32) {
        if let Some(callback) = self.idle_callbacks.iter_mut().find(|c| c.id == id) {
            callback.cancelled = true;
        }
    }

    /// Get idle callbacks that should run.
    pub fn get_ready_idle_callbacks(&mut self, idle_deadline: Instant) -> Vec<IdleCallback> {
        let now = Instant::now();
        let mut ready = Vec::new();
        let mut remaining = Vec::new();

        for callback in self.idle_callbacks.drain(..) {
            if callback.cancelled {
                continue;
            }

            // Check if timeout expired
            let timeout_expired = callback
                .timeout
                .map(|t| now.duration_since(callback.requested_at) >= t)
                .unwrap_or(false);

            if timeout_expired || now >= idle_deadline {
                ready.push(callback);
            } else {
                remaining.push(callback);
            }
        }

        self.idle_callbacks = remaining;
        ready
    }

    /// Check if the runtime has any pending work.
    pub fn has_pending_work(&self) -> bool {
        !self.timers.is_empty()
            || !self.microtasks.is_empty()
            || !self.macrotasks.is_empty()
            || !self.animation_frames.is_empty()
            || !self.idle_callbacks.is_empty()
    }

    /// Stop the runtime.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if the runtime is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the next timer deadline.
    pub fn next_timer_deadline(&self) -> Option<Instant> {
        self.timers
            .iter()
            .filter(|t| !t.cancelled)
            .map(|t| t.scheduled_at + t.delay)
            .min()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// A timer.
#[derive(Clone)]
pub struct Timer {
    /// Timer ID.
    pub id: u32,
    /// Callback to execute.
    pub callback: TimerCallback,
    /// When the timer was scheduled.
    pub scheduled_at: Instant,
    /// Delay before firing.
    pub delay: Duration,
    /// Whether to repeat.
    pub repeat: bool,
    /// Whether cancelled.
    pub cancelled: bool,
}

/// Timer callback type.
#[derive(Clone)]
pub enum TimerCallback {
    /// JavaScript function reference (stored as index into callback table).
    JsFunction(u64),
    /// Rust callback.
    Rust(Arc<dyn Fn() + Send + Sync>),
}

/// A microtask.
#[derive(Clone)]
pub struct Microtask {
    /// Task callback.
    pub callback: MicrotaskCallback,
}

/// Microtask callback type.
#[derive(Clone)]
pub enum MicrotaskCallback {
    /// JavaScript function reference.
    JsFunction(u64),
    /// Rust callback.
    Rust(Arc<dyn Fn() + Send + Sync>),
}

/// A macrotask.
#[derive(Clone)]
pub struct Macrotask {
    /// Task type.
    pub task_type: MacrotaskType,
    /// Task callback.
    pub callback: MacrotaskCallback,
}

/// Macrotask type.
#[derive(Clone, Debug)]
pub enum MacrotaskType {
    Timer,
    Event,
    MessageChannel,
    PostMessage,
    IO,
}

/// Macrotask callback type.
#[derive(Clone)]
pub enum MacrotaskCallback {
    /// JavaScript function reference.
    JsFunction(u64),
    /// Rust callback.
    Rust(Arc<dyn Fn() + Send + Sync>),
}

/// Animation frame callback.
pub struct AnimationFrameCallback {
    /// Callback ID.
    pub id: u32,
    /// Callback function.
    pub callback: AnimationFrameCallbackFn,
    /// Whether cancelled.
    pub cancelled: bool,
}

/// Animation frame callback function type.
pub type AnimationFrameCallbackFn = Arc<dyn Fn(f64) + Send + Sync>;

/// Idle callback.
pub struct IdleCallback {
    /// Callback ID.
    pub id: u32,
    /// Callback function.
    pub callback: IdleCallbackFn,
    /// Optional timeout.
    pub timeout: Option<Duration>,
    /// When requested.
    pub requested_at: Instant,
    /// Whether cancelled.
    pub cancelled: bool,
}

/// Idle callback function type.
pub type IdleCallbackFn = Arc<dyn Fn(IdleDeadline) + Send + Sync>;

/// Idle deadline information.
#[derive(Clone, Debug)]
pub struct IdleDeadline {
    /// Time remaining in this idle period.
    pub time_remaining: Duration,
    /// Whether the callback was triggered by timeout.
    pub did_timeout: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert!(runtime.is_running());
        assert!(!runtime.has_pending_work());
    }

    #[test]
    fn test_timer_scheduling() {
        let mut runtime = Runtime::new();
        let callback = TimerCallback::Rust(Arc::new(|| {}));

        let id = runtime.add_timer(callback, Duration::from_millis(100), false);
        assert!(runtime.has_pending_work());
        assert_eq!(id, 1);
    }

    #[test]
    fn test_timer_cancellation() {
        let mut runtime = Runtime::new();
        let callback = TimerCallback::Rust(Arc::new(|| {}));

        let id = runtime.add_timer(callback, Duration::from_secs(10), false);
        runtime.cancel_timer(id);

        // Timer still exists but is cancelled
        let ready = runtime.get_ready_timers();
        assert!(ready.is_empty());
    }

    #[test]
    fn test_microtask_queue() {
        let mut runtime = Runtime::new();

        runtime.queue_microtask(Microtask {
            callback: MicrotaskCallback::Rust(Arc::new(|| {})),
        });

        assert!(runtime.has_microtasks());
        let task = runtime.next_microtask();
        assert!(task.is_some());
        assert!(!runtime.has_microtasks());
    }
}
