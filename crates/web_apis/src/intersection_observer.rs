//! Intersection Observer API implementation.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Intersection Observer.
pub struct IntersectionObserver {
    /// Observer ID.
    id: u64,
    /// Callback function reference.
    callback: u64,
    /// Root element (None = viewport).
    root: Option<u64>,
    /// Root margin.
    root_margin: RootMargin,
    /// Thresholds.
    thresholds: Vec<f64>,
    /// Observed targets.
    targets: Vec<u64>,
    /// Whether the observer is active.
    active: bool,
}

impl IntersectionObserver {
    /// Create a new Intersection Observer.
    pub fn new(callback: u64, options: IntersectionObserverOptions) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };

        Self {
            id,
            callback,
            root: options.root,
            root_margin: options.root_margin,
            thresholds: Self::normalize_thresholds(options.threshold),
            targets: Vec::new(),
            active: true,
        }
    }

    /// Normalize thresholds to a sorted list.
    fn normalize_thresholds(threshold: Threshold) -> Vec<f64> {
        let mut thresholds = match threshold {
            Threshold::Single(t) => vec![t],
            Threshold::Multiple(ts) => ts,
        };

        // Clamp values between 0 and 1
        for t in &mut thresholds {
            *t = t.clamp(0.0, 1.0);
        }

        thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        thresholds
    }

    /// Observe a target element.
    pub fn observe(&mut self, target: u64) {
        if !self.targets.contains(&target) {
            self.targets.push(target);
        }
    }

    /// Stop observing a target element.
    pub fn unobserve(&mut self, target: u64) {
        self.targets.retain(|&t| t != target);
    }

    /// Stop observing all targets.
    pub fn disconnect(&mut self) {
        self.targets.clear();
        self.active = false;
    }

    /// Get the root element.
    pub fn root(&self) -> Option<u64> {
        self.root
    }

    /// Get the root margin.
    pub fn root_margin(&self) -> &RootMargin {
        &self.root_margin
    }

    /// Get the thresholds.
    pub fn thresholds(&self) -> &[f64] {
        &self.thresholds
    }

    /// Check if the observer is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get observed targets.
    pub fn targets(&self) -> &[u64] {
        &self.targets
    }

    /// Take records (for manual invocation).
    pub fn take_records(&self) -> Vec<IntersectionObserverEntry> {
        // Would return pending entries
        Vec::new()
    }
}

/// Intersection Observer options.
#[derive(Clone, Debug)]
pub struct IntersectionObserverOptions {
    /// Root element (None = viewport).
    pub root: Option<u64>,
    /// Root margin.
    pub root_margin: RootMargin,
    /// Threshold(s).
    pub threshold: Threshold,
}

impl Default for IntersectionObserverOptions {
    fn default() -> Self {
        Self {
            root: None,
            root_margin: RootMargin::default(),
            threshold: Threshold::Single(0.0),
        }
    }
}

/// Root margin specification.
#[derive(Clone, Debug, Default)]
pub struct RootMargin {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,
}

impl RootMargin {
    /// Parse root margin from a string.
    pub fn parse(margin: &str) -> Result<Self, String> {
        let parts: Vec<&str> = margin.split_whitespace().collect();

        match parts.len() {
            0 => Ok(Self::default()),
            1 => {
                let value = MarginValue::parse(parts[0])?;
                Ok(Self {
                    top: value.clone(),
                    right: value.clone(),
                    bottom: value.clone(),
                    left: value,
                })
            }
            2 => {
                let vertical = MarginValue::parse(parts[0])?;
                let horizontal = MarginValue::parse(parts[1])?;
                Ok(Self {
                    top: vertical.clone(),
                    right: horizontal.clone(),
                    bottom: vertical,
                    left: horizontal,
                })
            }
            3 => {
                let top = MarginValue::parse(parts[0])?;
                let horizontal = MarginValue::parse(parts[1])?;
                let bottom = MarginValue::parse(parts[2])?;
                Ok(Self {
                    top,
                    right: horizontal.clone(),
                    bottom,
                    left: horizontal,
                })
            }
            4 => Ok(Self {
                top: MarginValue::parse(parts[0])?,
                right: MarginValue::parse(parts[1])?,
                bottom: MarginValue::parse(parts[2])?,
                left: MarginValue::parse(parts[3])?,
            }),
            _ => Err("Invalid root margin format".to_string()),
        }
    }

    /// Convert to string.
    pub fn to_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.top, self.right, self.bottom, self.left
        )
    }
}

/// Margin value (pixels or percentage).
#[derive(Clone, Debug)]
pub enum MarginValue {
    Pixels(f64),
    Percentage(f64),
}

impl MarginValue {
    /// Parse a margin value.
    pub fn parse(value: &str) -> Result<Self, String> {
        let value = value.trim();

        if value.ends_with('%') {
            let num = value[..value.len() - 1]
                .parse::<f64>()
                .map_err(|_| "Invalid percentage")?;
            Ok(MarginValue::Percentage(num))
        } else if value.ends_with("px") {
            let num = value[..value.len() - 2]
                .parse::<f64>()
                .map_err(|_| "Invalid pixel value")?;
            Ok(MarginValue::Pixels(num))
        } else {
            // Assume pixels
            let num = value.parse::<f64>().map_err(|_| "Invalid value")?;
            Ok(MarginValue::Pixels(num))
        }
    }

    /// Get the value in pixels given a reference size.
    pub fn to_pixels(&self, reference: f64) -> f64 {
        match self {
            MarginValue::Pixels(px) => *px,
            MarginValue::Percentage(pct) => reference * pct / 100.0,
        }
    }
}

impl Default for MarginValue {
    fn default() -> Self {
        MarginValue::Pixels(0.0)
    }
}

impl std::fmt::Display for MarginValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarginValue::Pixels(px) => write!(f, "{}px", px),
            MarginValue::Percentage(pct) => write!(f, "{}%", pct),
        }
    }
}

/// Threshold specification.
#[derive(Clone, Debug)]
pub enum Threshold {
    Single(f64),
    Multiple(Vec<f64>),
}

/// Intersection observer entry.
#[derive(Clone, Debug)]
pub struct IntersectionObserverEntry {
    /// Target element.
    pub target: u64,
    /// Bounding client rect.
    pub bounding_client_rect: DOMRect,
    /// Intersection rect.
    pub intersection_rect: DOMRect,
    /// Root bounds.
    pub root_bounds: Option<DOMRect>,
    /// Intersection ratio.
    pub intersection_ratio: f64,
    /// Is intersecting.
    pub is_intersecting: bool,
    /// Time.
    pub time: f64,
}

/// DOMRect for intersection calculations.
#[derive(Clone, Debug, Default)]
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl DOMRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn top(&self) -> f64 {
        self.y
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn left(&self) -> f64 {
        self.x
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Calculate intersection with another rect.
    pub fn intersection(&self, other: &DOMRect) -> Option<DOMRect> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if left < right && top < bottom {
            Some(DOMRect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }

    /// Calculate area.
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// Intersection observer controller.
pub struct IntersectionObserverController {
    /// Active observers.
    observers: HashMap<u64, IntersectionObserver>,
    /// Observer ID counter.
    counter: u64,
}

impl IntersectionObserverController {
    pub fn new() -> Self {
        Self {
            observers: HashMap::new(),
            counter: 0,
        }
    }

    /// Create and register an observer.
    pub fn create_observer(
        &mut self,
        callback: u64,
        options: IntersectionObserverOptions,
    ) -> u64 {
        self.counter += 1;
        let observer = IntersectionObserver::new(callback, options);
        let id = observer.id;
        self.observers.insert(id, observer);
        id
    }

    /// Get an observer by ID.
    pub fn get(&self, id: u64) -> Option<&IntersectionObserver> {
        self.observers.get(&id)
    }

    /// Get a mutable observer by ID.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut IntersectionObserver> {
        self.observers.get_mut(&id)
    }

    /// Remove an observer.
    pub fn remove(&mut self, id: u64) {
        self.observers.remove(&id);
    }

    /// Process all observers (called on layout changes).
    pub fn process(&mut self, viewport: &DOMRect, elements: &HashMap<u64, DOMRect>) {
        for observer in self.observers.values() {
            if !observer.is_active() {
                continue;
            }

            let root_bounds = match observer.root {
                Some(root_id) => elements.get(&root_id).cloned(),
                None => Some(viewport.clone()),
            };

            let Some(root) = root_bounds else {
                continue;
            };

            // Apply root margin
            let expanded_root = self.apply_root_margin(&root, &observer.root_margin);

            for &target_id in &observer.targets {
                if let Some(target_rect) = elements.get(&target_id) {
                    let _entry = self.compute_entry(
                        target_id,
                        target_rect,
                        &expanded_root,
                        &observer.thresholds,
                    );
                    // Would invoke callback with entry
                }
            }
        }
    }

    fn apply_root_margin(&self, root: &DOMRect, margin: &RootMargin) -> DOMRect {
        let top = margin.top.to_pixels(root.height);
        let right = margin.right.to_pixels(root.width);
        let bottom = margin.bottom.to_pixels(root.height);
        let left = margin.left.to_pixels(root.width);

        DOMRect::new(
            root.x - left,
            root.y - top,
            root.width + left + right,
            root.height + top + bottom,
        )
    }

    fn compute_entry(
        &self,
        target: u64,
        target_rect: &DOMRect,
        root_bounds: &DOMRect,
        _thresholds: &[f64],
    ) -> IntersectionObserverEntry {
        let intersection = target_rect.intersection(root_bounds);

        let intersection_ratio = match &intersection {
            Some(int_rect) if target_rect.area() > 0.0 => {
                int_rect.area() / target_rect.area()
            }
            _ => 0.0,
        };

        IntersectionObserverEntry {
            target,
            bounding_client_rect: target_rect.clone(),
            intersection_rect: intersection.unwrap_or_default(),
            root_bounds: Some(root_bounds.clone()),
            intersection_ratio,
            is_intersecting: intersection_ratio > 0.0,
            time: 0.0, // Would use performance.now()
        }
    }
}

impl Default for IntersectionObserverController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_margin_parse() {
        let margin = RootMargin::parse("10px").unwrap();
        assert!(matches!(margin.top, MarginValue::Pixels(10.0)));

        let margin = RootMargin::parse("10px 20px").unwrap();
        assert!(matches!(margin.top, MarginValue::Pixels(10.0)));
        assert!(matches!(margin.right, MarginValue::Pixels(20.0)));

        let margin = RootMargin::parse("10%").unwrap();
        assert!(matches!(margin.top, MarginValue::Percentage(10.0)));
    }

    #[test]
    fn test_dom_rect_intersection() {
        let a = DOMRect::new(0.0, 0.0, 100.0, 100.0);
        let b = DOMRect::new(50.0, 50.0, 100.0, 100.0);

        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection.x, 50.0);
        assert_eq!(intersection.y, 50.0);
        assert_eq!(intersection.width, 50.0);
        assert_eq!(intersection.height, 50.0);
    }

    #[test]
    fn test_no_intersection() {
        let a = DOMRect::new(0.0, 0.0, 100.0, 100.0);
        let b = DOMRect::new(200.0, 200.0, 100.0, 100.0);

        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn test_intersection_observer() {
        let mut observer = IntersectionObserver::new(
            1,
            IntersectionObserverOptions {
                threshold: Threshold::Multiple(vec![0.0, 0.5, 1.0]),
                ..Default::default()
            },
        );

        observer.observe(100);
        observer.observe(101);
        assert_eq!(observer.targets().len(), 2);

        observer.unobserve(100);
        assert_eq!(observer.targets().len(), 1);

        observer.disconnect();
        assert!(observer.targets().is_empty());
        assert!(!observer.is_active());
    }
}
