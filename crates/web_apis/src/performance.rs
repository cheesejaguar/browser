//! Performance API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Performance API implementation.
pub struct Performance {
    /// Time origin (when the page started loading).
    time_origin: Instant,
    /// High resolution time offset.
    time_origin_timestamp: f64,
    /// Performance entries.
    entries: VecDeque<PerformanceEntry>,
    /// Maximum entries per type.
    max_entries_per_type: usize,
    /// User marks.
    marks: Vec<PerformanceMark>,
    /// User measures.
    measures: Vec<PerformanceMeasure>,
    /// Resource timing buffer size.
    resource_timing_buffer_size: usize,
}

impl Performance {
    /// Create a new Performance instance.
    pub fn new() -> Self {
        Self {
            time_origin: Instant::now(),
            time_origin_timestamp: Self::get_system_time_ms(),
            entries: VecDeque::new(),
            max_entries_per_type: 150,
            marks: Vec::new(),
            measures: Vec::new(),
            resource_timing_buffer_size: 250,
        }
    }

    /// Get the current system time in milliseconds.
    fn get_system_time_ms() -> f64 {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }

    /// Get high resolution time since time origin.
    pub fn now(&self) -> f64 {
        self.time_origin.elapsed().as_secs_f64() * 1000.0
    }

    /// Get the time origin as a Unix timestamp.
    pub fn time_origin(&self) -> f64 {
        self.time_origin_timestamp
    }

    /// Create a mark.
    pub fn mark(&mut self, name: &str) -> PerformanceMark {
        let mark = PerformanceMark {
            name: name.to_string(),
            entry_type: "mark".to_string(),
            start_time: self.now(),
            duration: 0.0,
            detail: None,
        };

        self.marks.push(mark.clone());
        mark
    }

    /// Create a mark with options.
    pub fn mark_with_options(&mut self, name: &str, start_time: Option<f64>) -> PerformanceMark {
        let mark = PerformanceMark {
            name: name.to_string(),
            entry_type: "mark".to_string(),
            start_time: start_time.unwrap_or_else(|| self.now()),
            duration: 0.0,
            detail: None,
        };

        self.marks.push(mark.clone());
        mark
    }

    /// Measure between two marks.
    pub fn measure(&mut self, name: &str, start_mark: Option<&str>, end_mark: Option<&str>) -> Option<PerformanceMeasure> {
        let start_time = match start_mark {
            Some(mark_name) => self.marks.iter().find(|m| m.name == mark_name)?.start_time,
            None => 0.0,
        };

        let end_time = match end_mark {
            Some(mark_name) => self.marks.iter().find(|m| m.name == mark_name)?.start_time,
            None => self.now(),
        };

        let measure = PerformanceMeasure {
            name: name.to_string(),
            entry_type: "measure".to_string(),
            start_time,
            duration: end_time - start_time,
            detail: None,
        };

        self.measures.push(measure.clone());
        Some(measure)
    }

    /// Get entries by type.
    pub fn get_entries_by_type(&self, entry_type: &str) -> Vec<PerformanceEntry> {
        let mut entries: Vec<PerformanceEntry> = self
            .entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .cloned()
            .collect();

        // Add marks
        if entry_type == "mark" {
            for mark in &self.marks {
                entries.push(PerformanceEntry::Mark(mark.clone()));
            }
        }

        // Add measures
        if entry_type == "measure" {
            for measure in &self.measures {
                entries.push(PerformanceEntry::Measure(measure.clone()));
            }
        }

        entries
    }

    /// Get entries by name.
    pub fn get_entries_by_name(&self, name: &str, entry_type: Option<&str>) -> Vec<PerformanceEntry> {
        let mut entries: Vec<PerformanceEntry> = self
            .entries
            .iter()
            .filter(|e| {
                e.name() == name && entry_type.map(|t| e.entry_type == t).unwrap_or(true)
            })
            .cloned()
            .collect();

        // Add matching marks
        for mark in &self.marks {
            if mark.name == name && entry_type.map(|t| t == "mark").unwrap_or(true) {
                entries.push(PerformanceEntry::Mark(mark.clone()));
            }
        }

        // Add matching measures
        for measure in &self.measures {
            if measure.name == name && entry_type.map(|t| t == "measure").unwrap_or(true) {
                entries.push(PerformanceEntry::Measure(measure.clone()));
            }
        }

        entries
    }

    /// Clear marks.
    pub fn clear_marks(&mut self, name: Option<&str>) {
        match name {
            Some(n) => self.marks.retain(|m| m.name != n),
            None => self.marks.clear(),
        }
    }

    /// Clear measures.
    pub fn clear_measures(&mut self, name: Option<&str>) {
        match name {
            Some(n) => self.measures.retain(|m| m.name != n),
            None => self.measures.clear(),
        }
    }

    /// Clear resource timings.
    pub fn clear_resource_timings(&mut self) {
        self.entries.retain(|e| e.entry_type != "resource");
    }

    /// Set resource timing buffer size.
    pub fn set_resource_timing_buffer_size(&mut self, size: usize) {
        self.resource_timing_buffer_size = size;
    }

    /// Add a resource timing entry.
    pub fn add_resource_timing(&mut self, entry: ResourceTiming) {
        // Check buffer size
        let resource_count = self.entries.iter().filter(|e| e.entry_type == "resource").count();
        if resource_count >= self.resource_timing_buffer_size {
            // Remove oldest resource entry
            if let Some(pos) = self.entries.iter().position(|e| e.entry_type == "resource") {
                self.entries.remove(pos);
            }
        }

        self.entries.push_back(PerformanceEntry::Resource(entry));
    }

    /// Add a navigation timing entry.
    pub fn set_navigation_timing(&mut self, entry: NavigationTiming) {
        // Remove existing navigation entry
        self.entries.retain(|e| e.entry_type != "navigation");
        self.entries.push_front(PerformanceEntry::Navigation(entry));
    }

    /// Register the Performance API on the global object.
    pub fn register(performance: Arc<RwLock<Performance>>, context: &mut Context) {
        let perf = performance.read();
        let time_origin = perf.time_origin();
        drop(perf);

        let performance_obj = ObjectInitializer::new(context)
            .property(js_string!("timeOrigin"), time_origin, Attribute::READONLY)
            .function(NativeFunction::from_fn_ptr(performance_now), js_string!("now"), 0)
            .function(NativeFunction::from_fn_ptr(performance_mark), js_string!("mark"), 1)
            .function(NativeFunction::from_fn_ptr(performance_measure), js_string!("measure"), 3)
            .function(NativeFunction::from_fn_ptr(performance_clear_marks), js_string!("clearMarks"), 1)
            .function(NativeFunction::from_fn_ptr(performance_clear_measures), js_string!("clearMeasures"), 1)
            .function(NativeFunction::from_fn_ptr(performance_clear_resource_timings), js_string!("clearResourceTimings"), 0)
            .function(NativeFunction::from_fn_ptr(performance_get_entries), js_string!("getEntries"), 0)
            .function(NativeFunction::from_fn_ptr(performance_get_entries_by_type), js_string!("getEntriesByType"), 1)
            .function(NativeFunction::from_fn_ptr(performance_get_entries_by_name), js_string!("getEntriesByName"), 2)
            .function(NativeFunction::from_fn_ptr(performance_set_resource_timing_buffer_size), js_string!("setResourceTimingBufferSize"), 1)
            .function(NativeFunction::from_fn_ptr(performance_to_json), js_string!("toJSON"), 0)
            .build();

        context
            .register_global_property(js_string!("performance"), performance_obj, Attribute::all())
            .expect("Failed to register performance");
    }
}

impl Default for Performance {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance entry types.
#[derive(Clone, Debug)]
pub enum PerformanceEntry {
    Mark(PerformanceMark),
    Measure(PerformanceMeasure),
    Resource(ResourceTiming),
    Navigation(NavigationTiming),
    Paint(PaintTiming),
    LongTask(LongTaskTiming),
}

impl PerformanceEntry {
    pub fn name(&self) -> &str {
        match self {
            PerformanceEntry::Mark(m) => &m.name,
            PerformanceEntry::Measure(m) => &m.name,
            PerformanceEntry::Resource(r) => &r.name,
            PerformanceEntry::Navigation(n) => &n.name,
            PerformanceEntry::Paint(p) => &p.name,
            PerformanceEntry::LongTask(l) => &l.name,
        }
    }

    pub fn entry_type(&self) -> &str {
        match self {
            PerformanceEntry::Mark(_) => "mark",
            PerformanceEntry::Measure(_) => "measure",
            PerformanceEntry::Resource(_) => "resource",
            PerformanceEntry::Navigation(_) => "navigation",
            PerformanceEntry::Paint(_) => "paint",
            PerformanceEntry::LongTask(_) => "longtask",
        }
    }
}

/// Performance mark.
#[derive(Clone, Debug)]
pub struct PerformanceMark {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub detail: Option<serde_json::Value>,
}

/// Performance measure.
#[derive(Clone, Debug)]
pub struct PerformanceMeasure {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub detail: Option<serde_json::Value>,
}

/// Resource timing entry.
#[derive(Clone, Debug)]
pub struct ResourceTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub initiator_type: String,
    pub next_hop_protocol: String,
    pub worker_start: f64,
    pub redirect_start: f64,
    pub redirect_end: f64,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub connect_end: f64,
    pub secure_connection_start: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
}

/// Navigation timing entry.
#[derive(Clone, Debug)]
pub struct NavigationTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub navigation_type: NavigationType,
    pub unload_event_start: f64,
    pub unload_event_end: f64,
    pub dom_interactive: f64,
    pub dom_content_loaded_event_start: f64,
    pub dom_content_loaded_event_end: f64,
    pub dom_complete: f64,
    pub load_event_start: f64,
    pub load_event_end: f64,
    pub redirect_count: u32,
}

/// Navigation type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationType {
    Navigate,
    Reload,
    BackForward,
    Prerender,
}

/// Paint timing entry.
#[derive(Clone, Debug)]
pub struct PaintTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
}

/// Long task timing entry.
#[derive(Clone, Debug)]
pub struct LongTaskTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub attribution: Vec<TaskAttribution>,
}

/// Task attribution for long tasks.
#[derive(Clone, Debug)]
pub struct TaskAttribution {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub container_type: String,
    pub container_src: String,
    pub container_id: String,
    pub container_name: String,
}

// Native function implementations
fn performance_now(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    // Would use the actual Performance instance
    Ok(JsValue::from(0.0))
}

fn performance_mark(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(context)?;
    Ok(JsValue::undefined())
}

fn performance_measure(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(context)?;
    let _start_mark = args.get(1);
    let _end_mark = args.get(2);
    Ok(JsValue::undefined())
}

fn performance_clear_marks(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _name = args.get(0);
    Ok(JsValue::undefined())
}

fn performance_clear_measures(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _name = args.get(0);
    Ok(JsValue::undefined())
}

fn performance_clear_resource_timings(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn performance_get_entries(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined()) // Would return array
}

fn performance_get_entries_by_type(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _entry_type = args.get_or_undefined(0).to_string(context)?;
    Ok(JsValue::undefined()) // Would return array
}

fn performance_get_entries_by_name(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _name = args.get_or_undefined(0).to_string(context)?;
    let _entry_type = args.get(1);
    Ok(JsValue::undefined()) // Would return array
}

fn performance_set_resource_timing_buffer_size(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _size = args.get_or_undefined(0).to_u32(context)?;
    Ok(JsValue::undefined())
}

fn performance_to_json(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined()) // Would return JSON object
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_now() {
        let perf = Performance::new();
        let now1 = perf.now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let now2 = perf.now();
        assert!(now2 > now1);
    }

    #[test]
    fn test_performance_marks() {
        let mut perf = Performance::new();

        perf.mark("start");
        std::thread::sleep(std::time::Duration::from_millis(10));
        perf.mark("end");

        let entries = perf.get_entries_by_type("mark");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_performance_measure() {
        let mut perf = Performance::new();

        perf.mark("start");
        std::thread::sleep(std::time::Duration::from_millis(10));
        perf.mark("end");

        let measure = perf.measure("duration", Some("start"), Some("end")).unwrap();
        assert!(measure.duration >= 10.0);
    }

    #[test]
    fn test_performance_clear() {
        let mut perf = Performance::new();

        perf.mark("test1");
        perf.mark("test2");
        assert_eq!(perf.marks.len(), 2);

        perf.clear_marks(Some("test1"));
        assert_eq!(perf.marks.len(), 1);

        perf.clear_marks(None);
        assert_eq!(perf.marks.len(), 0);
    }
}
