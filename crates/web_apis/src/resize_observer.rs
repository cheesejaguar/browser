//! Resize Observer API implementation.

use std::collections::HashMap;

/// Resize Observer.
pub struct ResizeObserver {
    /// Observer ID.
    id: u64,
    /// Callback function reference.
    callback: u64,
    /// Observed targets.
    targets: HashMap<u64, ResizeObserverOptions>,
    /// Whether the observer is active.
    active: bool,
}

impl ResizeObserver {
    /// Create a new Resize Observer.
    pub fn new(callback: u64) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };

        Self {
            id,
            callback,
            targets: HashMap::new(),
            active: true,
        }
    }

    /// Observe a target element.
    pub fn observe(&mut self, target: u64, options: Option<ResizeObserverOptions>) {
        self.targets.insert(target, options.unwrap_or_default());
    }

    /// Stop observing a target element.
    pub fn unobserve(&mut self, target: u64) {
        self.targets.remove(&target);
    }

    /// Stop observing all targets.
    pub fn disconnect(&mut self) {
        self.targets.clear();
        self.active = false;
    }

    /// Check if observing a target.
    pub fn is_observing(&self, target: u64) -> bool {
        self.targets.contains_key(&target)
    }

    /// Get observed targets.
    pub fn targets(&self) -> impl Iterator<Item = &u64> {
        self.targets.keys()
    }

    /// Get the callback.
    pub fn callback(&self) -> u64 {
        self.callback
    }

    /// Check if active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Resize observer options.
#[derive(Clone, Debug)]
pub struct ResizeObserverOptions {
    /// Box to observe.
    pub box_type: ResizeObserverBoxOptions,
}

impl Default for ResizeObserverOptions {
    fn default() -> Self {
        Self {
            box_type: ResizeObserverBoxOptions::ContentBox,
        }
    }
}

/// Box options for resize observer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeObserverBoxOptions {
    ContentBox,
    BorderBox,
    DevicePixelContentBox,
}

/// Resize observer entry.
#[derive(Clone, Debug)]
pub struct ResizeObserverEntry {
    /// Target element.
    pub target: u64,
    /// Content rect.
    pub content_rect: DOMRectReadOnly,
    /// Border box size.
    pub border_box_size: Vec<ResizeObserverSize>,
    /// Content box size.
    pub content_box_size: Vec<ResizeObserverSize>,
    /// Device pixel content box size.
    pub device_pixel_content_box_size: Vec<ResizeObserverSize>,
}

/// Size reported by resize observer.
#[derive(Clone, Debug)]
pub struct ResizeObserverSize {
    /// Inline size (width in horizontal writing mode).
    pub inline_size: f64,
    /// Block size (height in horizontal writing mode).
    pub block_size: f64,
}

impl ResizeObserverSize {
    pub fn new(inline_size: f64, block_size: f64) -> Self {
        Self {
            inline_size,
            block_size,
        }
    }
}

/// DOMRectReadOnly for resize observer.
#[derive(Clone, Debug, Default)]
pub struct DOMRectReadOnly {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl DOMRectReadOnly {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn top(&self) -> f64 {
        self.y
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn left(&self) -> f64 {
        self.x
    }
}

/// Resize observer controller.
pub struct ResizeObserverController {
    /// Active observers.
    observers: Vec<ResizeObserver>,
    /// Previous sizes for change detection.
    previous_sizes: HashMap<u64, (f64, f64)>,
}

impl ResizeObserverController {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            previous_sizes: HashMap::new(),
        }
    }

    /// Register an observer.
    pub fn register(&mut self, observer: ResizeObserver) {
        self.observers.push(observer);
    }

    /// Process resize observations.
    pub fn process(&mut self, elements: &HashMap<u64, ElementSize>) -> Vec<(u64, Vec<ResizeObserverEntry>)> {
        let mut notifications = Vec::new();

        for observer in &self.observers {
            if !observer.is_active() {
                continue;
            }

            let mut entries = Vec::new();

            for (&target, options) in &observer.targets {
                let Some(size) = elements.get(&target) else {
                    continue;
                };

                // Check if size changed
                let (width, height) = match options.box_type {
                    ResizeObserverBoxOptions::ContentBox => {
                        (size.content_width, size.content_height)
                    }
                    ResizeObserverBoxOptions::BorderBox => {
                        (size.border_box_width, size.border_box_height)
                    }
                    ResizeObserverBoxOptions::DevicePixelContentBox => {
                        (size.device_pixel_content_width, size.device_pixel_content_height)
                    }
                };

                let previous = self.previous_sizes.get(&target);
                let changed = previous.map(|&(w, h)| (w - width).abs() > 0.01 || (h - height).abs() > 0.01).unwrap_or(true);

                if changed {
                    self.previous_sizes.insert(target, (width, height));

                    entries.push(ResizeObserverEntry {
                        target,
                        content_rect: DOMRectReadOnly::new(
                            0.0,
                            0.0,
                            size.content_width,
                            size.content_height,
                        ),
                        border_box_size: vec![ResizeObserverSize::new(
                            size.border_box_width,
                            size.border_box_height,
                        )],
                        content_box_size: vec![ResizeObserverSize::new(
                            size.content_width,
                            size.content_height,
                        )],
                        device_pixel_content_box_size: vec![ResizeObserverSize::new(
                            size.device_pixel_content_width,
                            size.device_pixel_content_height,
                        )],
                    });
                }
            }

            if !entries.is_empty() {
                notifications.push((observer.callback, entries));
            }
        }

        notifications
    }
}

impl Default for ResizeObserverController {
    fn default() -> Self {
        Self::new()
    }
}

/// Element size information.
#[derive(Clone, Debug)]
pub struct ElementSize {
    pub content_width: f64,
    pub content_height: f64,
    pub border_box_width: f64,
    pub border_box_height: f64,
    pub device_pixel_content_width: f64,
    pub device_pixel_content_height: f64,
}

impl ElementSize {
    pub fn new(content_width: f64, content_height: f64, device_pixel_ratio: f64) -> Self {
        Self {
            content_width,
            content_height,
            border_box_width: content_width,
            border_box_height: content_height,
            device_pixel_content_width: content_width * device_pixel_ratio,
            device_pixel_content_height: content_height * device_pixel_ratio,
        }
    }

    pub fn with_border_box(mut self, width: f64, height: f64) -> Self {
        self.border_box_width = width;
        self.border_box_height = height;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_observer_creation() {
        let observer = ResizeObserver::new(1);
        assert!(observer.is_active());
        assert_eq!(observer.callback(), 1);
    }

    #[test]
    fn test_resize_observer_observe() {
        let mut observer = ResizeObserver::new(1);

        observer.observe(100, None);
        assert!(observer.is_observing(100));

        observer.observe(
            101,
            Some(ResizeObserverOptions {
                box_type: ResizeObserverBoxOptions::BorderBox,
            }),
        );
        assert!(observer.is_observing(101));

        observer.unobserve(100);
        assert!(!observer.is_observing(100));
        assert!(observer.is_observing(101));
    }

    #[test]
    fn test_resize_observer_disconnect() {
        let mut observer = ResizeObserver::new(1);
        observer.observe(100, None);
        observer.observe(101, None);

        observer.disconnect();
        assert!(!observer.is_active());
        assert!(!observer.is_observing(100));
        assert!(!observer.is_observing(101));
    }

    #[test]
    fn test_resize_observer_size() {
        let size = ResizeObserverSize::new(100.0, 200.0);
        assert_eq!(size.inline_size, 100.0);
        assert_eq!(size.block_size, 200.0);
    }

    #[test]
    fn test_element_size() {
        let size = ElementSize::new(100.0, 200.0, 2.0);
        assert_eq!(size.content_width, 100.0);
        assert_eq!(size.content_height, 200.0);
        assert_eq!(size.device_pixel_content_width, 200.0);
        assert_eq!(size.device_pixel_content_height, 400.0);
    }
}
