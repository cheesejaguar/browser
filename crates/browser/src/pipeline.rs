//! Render pipeline - coordinates the rendering stages.

use std::sync::Arc;

/// Render pipeline stages.
pub struct RenderPipeline {
    /// Pipeline stages.
    stages: Vec<PipelineStage>,
    /// Current frame.
    frame: u64,
}

impl RenderPipeline {
    /// Create a new render pipeline.
    pub fn new() -> Self {
        Self {
            stages: vec![
                PipelineStage::Parse,
                PipelineStage::Style,
                PipelineStage::Layout,
                PipelineStage::Paint,
                PipelineStage::Composite,
            ],
            frame: 0,
        }
    }

    /// Run the pipeline.
    pub fn run(&mut self, document: &DocumentSnapshot) -> PipelineResult {
        let mut result = PipelineResult::new();

        for stage in &self.stages {
            let stage_result = self.run_stage(*stage, document);
            result.stage_times.push((*stage, stage_result.duration));

            if !stage_result.success {
                result.success = false;
                result.error = stage_result.error;
                break;
            }
        }

        self.frame += 1;
        result.frame = self.frame;
        result
    }

    fn run_stage(&self, stage: PipelineStage, _document: &DocumentSnapshot) -> StageResult {
        let start = std::time::Instant::now();

        // Each stage would invoke the appropriate crate
        let success = match stage {
            PipelineStage::Parse => {
                // Would use html_parser crate
                true
            }
            PipelineStage::Style => {
                // Would use style crate
                true
            }
            PipelineStage::Layout => {
                // Would use layout crate
                true
            }
            PipelineStage::Paint => {
                // Would use render crate
                true
            }
            PipelineStage::Composite => {
                // Would use compositor crate
                true
            }
        };

        StageResult {
            success,
            duration: start.elapsed(),
            error: None,
        }
    }

    /// Get current frame number.
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Invalidate and request repaint.
    pub fn invalidate(&mut self) {
        // Would trigger repaint
    }

    /// Invalidate specific stage.
    pub fn invalidate_stage(&mut self, _stage: PipelineStage) {
        // Would trigger repaint from that stage
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineStage {
    /// Parse HTML/CSS.
    Parse,
    /// Compute styles.
    Style,
    /// Perform layout.
    Layout,
    /// Paint to display list.
    Paint,
    /// Composite layers.
    Composite,
}

/// Result of running the pipeline.
#[derive(Debug)]
pub struct PipelineResult {
    /// Whether the pipeline succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Frame number.
    pub frame: u64,
    /// Time spent in each stage.
    pub stage_times: Vec<(PipelineStage, std::time::Duration)>,
}

impl PipelineResult {
    /// Create a new successful result.
    pub fn new() -> Self {
        Self {
            success: true,
            error: None,
            frame: 0,
            stage_times: Vec::new(),
        }
    }

    /// Get total time.
    pub fn total_time(&self) -> std::time::Duration {
        self.stage_times.iter().map(|(_, d)| *d).sum()
    }
}

impl Default for PipelineResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single stage.
#[derive(Debug)]
struct StageResult {
    success: bool,
    duration: std::time::Duration,
    error: Option<String>,
}

/// Document snapshot for rendering.
#[derive(Debug)]
pub struct DocumentSnapshot {
    /// HTML content.
    pub html: String,
    /// Stylesheets.
    pub stylesheets: Vec<String>,
    /// Viewport width.
    pub viewport_width: u32,
    /// Viewport height.
    pub viewport_height: u32,
}

impl DocumentSnapshot {
    /// Create a new snapshot.
    pub fn new(html: &str, viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            html: html.to_string(),
            stylesheets: Vec::new(),
            viewport_width,
            viewport_height,
        }
    }

    /// Add a stylesheet.
    pub fn add_stylesheet(&mut self, css: &str) {
        self.stylesheets.push(css.to_string());
    }
}

/// Frame timing information.
#[derive(Clone, Copy, Debug)]
pub struct FrameTiming {
    /// Frame number.
    pub frame: u64,
    /// Frame start time.
    pub start: std::time::Instant,
    /// Frame duration.
    pub duration: std::time::Duration,
    /// Parse time.
    pub parse_time: std::time::Duration,
    /// Style time.
    pub style_time: std::time::Duration,
    /// Layout time.
    pub layout_time: std::time::Duration,
    /// Paint time.
    pub paint_time: std::time::Duration,
    /// Composite time.
    pub composite_time: std::time::Duration,
}

impl FrameTiming {
    /// Get FPS from duration.
    pub fn fps(&self) -> f64 {
        1.0 / self.duration.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = RenderPipeline::new();
        assert_eq!(pipeline.frame(), 0);
    }

    #[test]
    fn test_pipeline_run() {
        let mut pipeline = RenderPipeline::new();
        let document = DocumentSnapshot::new("<html><body>Hello</body></html>", 800, 600);

        let result = pipeline.run(&document);
        assert!(result.success);
        assert_eq!(result.frame, 1);
    }
}
