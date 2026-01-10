//! JavaScript event loop implementation.

use crate::engine::JsEngine;
use crate::runtime::{Macrotask, MacrotaskType, Microtask, Runtime, Timer};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Event loop for executing JavaScript.
pub struct EventLoop {
    /// JavaScript engine.
    engine: JsEngine,
    /// Whether the loop is running.
    running: bool,
    /// Frame rate target (for requestAnimationFrame).
    target_frame_time: Duration,
    /// Last frame time.
    last_frame_time: Instant,
}

impl EventLoop {
    /// Create a new event loop.
    pub fn new() -> Self {
        Self {
            engine: JsEngine::new(),
            running: false,
            target_frame_time: Duration::from_secs_f64(1.0 / 60.0), // 60 FPS
            last_frame_time: Instant::now(),
        }
    }

    /// Create with an existing engine.
    pub fn with_engine(engine: JsEngine) -> Self {
        Self {
            engine,
            running: false,
            target_frame_time: Duration::from_secs_f64(1.0 / 60.0),
            last_frame_time: Instant::now(),
        }
    }

    /// Run the event loop until there's no more work.
    pub fn run(&mut self) {
        self.running = true;

        while self.running && self.has_pending_work() {
            self.tick();
        }

        self.running = false;
    }

    /// Run a single iteration of the event loop.
    pub fn tick(&mut self) {
        // 1. Run all microtasks
        self.drain_microtasks();

        // 2. Check for ready timers
        self.process_timers();

        // 3. Process a macrotask
        self.process_macrotask();

        // 4. Run all microtasks again (timers/macrotasks may have queued some)
        self.drain_microtasks();

        // 5. Check if we need to run animation frames
        self.maybe_run_animation_frames();

        // 6. Run pending jobs from the JS engine
        self.engine.run_pending_jobs();
    }

    /// Drain all microtasks.
    fn drain_microtasks(&mut self) {
        let runtime = self.engine.runtime();

        loop {
            let task = runtime.write().next_microtask();
            match task {
                Some(microtask) => {
                    self.execute_microtask(microtask);
                }
                None => break,
            }
        }
    }

    /// Execute a microtask.
    fn execute_microtask(&mut self, task: Microtask) {
        match task.callback {
            crate::runtime::MicrotaskCallback::JsFunction(_callback_id) => {
                // Would call the JS function here
            }
            crate::runtime::MicrotaskCallback::Rust(callback) => {
                callback();
            }
        }
    }

    /// Process ready timers.
    fn process_timers(&mut self) {
        let runtime = self.engine.runtime();
        let ready_timers = runtime.write().get_ready_timers();

        for timer in ready_timers {
            self.execute_timer(timer);
        }
    }

    /// Execute a timer callback.
    fn execute_timer(&mut self, timer: Timer) {
        match timer.callback {
            crate::runtime::TimerCallback::JsFunction(_callback_id) => {
                // Would call the JS function here
            }
            crate::runtime::TimerCallback::Rust(callback) => {
                callback();
            }
        }
    }

    /// Process a single macrotask.
    fn process_macrotask(&mut self) {
        let runtime = self.engine.runtime();
        let task = runtime.write().next_macrotask();

        if let Some(macrotask) = task {
            self.execute_macrotask(macrotask);
        }
    }

    /// Execute a macrotask.
    fn execute_macrotask(&mut self, task: Macrotask) {
        match task.callback {
            crate::runtime::MacrotaskCallback::JsFunction(_callback_id) => {
                // Would call the JS function here
            }
            crate::runtime::MacrotaskCallback::Rust(callback) => {
                callback();
            }
        }
    }

    /// Maybe run animation frames (if enough time has passed).
    fn maybe_run_animation_frames(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame_time);

        if elapsed >= self.target_frame_time {
            self.run_animation_frames(now);
            self.last_frame_time = now;
        }
    }

    /// Run all pending animation frame callbacks.
    fn run_animation_frames(&mut self, now: Instant) {
        let runtime = self.engine.runtime();
        let callbacks = runtime.write().drain_animation_frames();

        // Calculate high-resolution timestamp (milliseconds since time origin)
        let timestamp = now.elapsed().as_secs_f64() * 1000.0;

        for callback in callbacks {
            (callback.callback)(timestamp);
        }
    }

    /// Check if there's pending work.
    fn has_pending_work(&self) -> bool {
        self.engine.runtime().read().has_pending_work()
    }

    /// Stop the event loop.
    pub fn stop(&mut self) {
        self.running = false;
        self.engine.runtime().write().stop();
    }

    /// Check if the event loop is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the engine.
    pub fn engine(&self) -> &JsEngine {
        &self.engine
    }

    /// Get mutable engine.
    pub fn engine_mut(&mut self) -> &mut JsEngine {
        &mut self.engine
    }

    /// Execute a script.
    pub fn execute(&mut self, source: &str) -> Result<boa_engine::JsValue, crate::engine::JsEngineError> {
        let result = self.engine.execute(source);

        // Run the event loop to process any queued tasks
        while self.has_pending_work() {
            self.tick();
        }

        result
    }

    /// Set the target frame rate.
    pub fn set_frame_rate(&mut self, fps: f64) {
        self.target_frame_time = Duration::from_secs_f64(1.0 / fps);
    }

    /// Get the next deadline (for integration with external event loops).
    pub fn next_deadline(&self) -> Option<Instant> {
        let runtime = self.engine.runtime();
        let runtime = runtime.read();

        // Get the nearest timer deadline
        let timer_deadline = runtime.next_timer_deadline();

        // Get the next animation frame deadline
        let frame_deadline = self.last_frame_time + self.target_frame_time;

        match timer_deadline {
            Some(t) => Some(t.min(frame_deadline)),
            None => Some(frame_deadline),
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Event loop integration for external schedulers.
pub trait EventLoopIntegration {
    /// Called when there's work to do.
    fn wake(&self);

    /// Get the current time.
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Simple wake implementation using a channel.
pub struct ChannelWaker {
    sender: std::sync::mpsc::Sender<()>,
}

impl ChannelWaker {
    pub fn new(sender: std::sync::mpsc::Sender<()>) -> Self {
        Self { sender }
    }
}

impl EventLoopIntegration for ChannelWaker {
    fn wake(&self) {
        let _ = self.sender.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_creation() {
        let loop_ = EventLoop::new();
        assert!(!loop_.is_running());
    }

    #[test]
    fn test_event_loop_tick() {
        let mut loop_ = EventLoop::new();
        loop_.tick(); // Should not panic even with no work
    }

    #[test]
    fn test_script_execution() {
        let mut loop_ = EventLoop::new();
        let result = loop_.execute("1 + 2").unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }
}
