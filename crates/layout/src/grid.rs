//! CSS Grid layout implementation.

use crate::box_model::ContainingBlock;
use crate::layout_box::{GridLayoutData, LayoutBox, LayoutBoxId};
use crate::tree::LayoutTree;
use style::computed::{
    AlignContent, AlignItems, ComputedStyle, GridAutoFlow, GridTemplateTrack, JustifyContent,
    JustifyItems, LengthPercentageAuto,
};

/// Grid formatting context.
pub struct GridFormattingContext {
    /// Explicit column tracks.
    columns: Vec<GridTrack>,
    /// Explicit row tracks.
    rows: Vec<GridTrack>,
    /// Column gap.
    column_gap: f32,
    /// Row gap.
    row_gap: f32,
    /// Auto flow direction.
    auto_flow: GridAutoFlow,
    /// Justify items.
    justify_items: JustifyItems,
    /// Align items.
    align_items: AlignItems,
    /// Justify content.
    justify_content: JustifyContent,
    /// Align content.
    align_content: AlignContent,
    /// Container width.
    container_width: f32,
    /// Container height.
    container_height: Option<f32>,
}

/// A grid track (row or column).
#[derive(Clone, Debug)]
struct GridTrack {
    /// Track sizing function.
    sizing: TrackSizing,
    /// Computed base size.
    base_size: f32,
    /// Growth limit.
    growth_limit: f32,
    /// Final size after distribution.
    final_size: f32,
    /// Start position.
    start: f32,
}

/// Track sizing function.
#[derive(Clone, Debug)]
enum TrackSizing {
    /// Fixed length.
    Fixed(f32),
    /// Percentage of container.
    Percentage(f32),
    /// Flexible fraction.
    Flex(f32),
    /// Minimum content size.
    MinContent,
    /// Maximum content size.
    MaxContent,
    /// Auto sizing.
    Auto,
    /// Fit-content with max.
    FitContent(f32),
    /// Min/max bounds.
    MinMax(Box<TrackSizing>, Box<TrackSizing>),
}

/// Grid item for placement.
#[derive(Clone, Debug)]
struct GridItem {
    box_id: LayoutBoxId,
    /// Column start (1-indexed, 0 = auto).
    column_start: i32,
    /// Column end (1-indexed, 0 = auto, negative = span).
    column_end: i32,
    /// Row start.
    row_start: i32,
    /// Row end.
    row_end: i32,
    /// Resolved column range.
    resolved_column: (usize, usize),
    /// Resolved row range.
    resolved_row: (usize, usize),
    /// Content width.
    content_width: f32,
    /// Content height.
    content_height: f32,
}

/// Grid cell occupation map.
struct OccupancyGrid {
    columns: usize,
    rows: usize,
    cells: Vec<bool>,
}

impl OccupancyGrid {
    fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns,
            rows,
            cells: vec![false; columns * rows],
        }
    }

    fn is_occupied(&self, col: usize, row: usize) -> bool {
        if col >= self.columns || row >= self.rows {
            return false;
        }
        self.cells[row * self.columns + col]
    }

    fn occupy(&mut self, col: usize, row: usize) {
        if col < self.columns && row < self.rows {
            self.cells[row * self.columns + col] = true;
        }
    }

    fn occupy_range(&mut self, col_start: usize, col_end: usize, row_start: usize, row_end: usize) {
        for row in row_start..row_end {
            for col in col_start..col_end {
                self.occupy(col, row);
            }
        }
    }

    fn find_empty_cell(&self, start_col: usize, start_row: usize) -> Option<(usize, usize)> {
        for row in start_row..self.rows {
            let col_start = if row == start_row { start_col } else { 0 };
            for col in col_start..self.columns {
                if !self.is_occupied(col, row) {
                    return Some((col, row));
                }
            }
        }
        None
    }

    fn can_place(&self, col: usize, row: usize, width: usize, height: usize) -> bool {
        if col + width > self.columns || row + height > self.rows {
            return false;
        }
        for r in row..(row + height) {
            for c in col..(col + width) {
                if self.is_occupied(c, r) {
                    return false;
                }
            }
        }
        true
    }

    fn expand_rows(&mut self, new_rows: usize) {
        if new_rows > self.rows {
            self.cells.resize(self.columns * new_rows, false);
            self.rows = new_rows;
        }
    }

    fn expand_columns(&mut self, new_cols: usize) {
        if new_cols > self.columns {
            let mut new_cells = vec![false; new_cols * self.rows];
            for row in 0..self.rows {
                for col in 0..self.columns {
                    new_cells[row * new_cols + col] = self.cells[row * self.columns + col];
                }
            }
            self.cells = new_cells;
            self.columns = new_cols;
        }
    }
}

impl GridFormattingContext {
    pub fn new(style: &ComputedStyle, width: f32, height: Option<f32>) -> Self {
        // Parse grid-template-columns
        let columns = Self::parse_track_list(&style.grid_template_columns, width);

        // Parse grid-template-rows
        let rows = Self::parse_track_list(&style.grid_template_rows, height.unwrap_or(0.0));

        // Get gaps
        let column_gap = Self::resolve_gap(&style.column_gap, width);
        let row_gap = Self::resolve_gap(&style.row_gap, height.unwrap_or(width));

        Self {
            columns,
            rows,
            column_gap,
            row_gap,
            auto_flow: style.grid_auto_flow.clone(),
            justify_items: style.justify_items.clone(),
            align_items: style.align_items.clone(),
            justify_content: style.justify_content.clone(),
            align_content: style.align_content.clone(),
            container_width: width,
            container_height: height,
        }
    }

    /// Parse track list from style.
    fn parse_track_list(template: &GridTemplateTrack, available: f32) -> Vec<GridTrack> {
        match template {
            GridTemplateTrack::None => Vec::new(),
            GridTemplateTrack::TrackList(tracks) => {
                tracks
                    .iter()
                    .map(|t| {
                        let sizing = Self::parse_track_sizing(t, available);
                        GridTrack {
                            sizing,
                            base_size: 0.0,
                            growth_limit: f32::INFINITY,
                            final_size: 0.0,
                            start: 0.0,
                        }
                    })
                    .collect()
            }
            GridTemplateTrack::Subgrid => Vec::new(), // Not fully supported
        }
    }

    /// Parse a single track sizing value.
    fn parse_track_sizing(value: &str, available: f32) -> TrackSizing {
        let value = value.trim();

        if value == "auto" {
            return TrackSizing::Auto;
        }

        if value == "min-content" {
            return TrackSizing::MinContent;
        }

        if value == "max-content" {
            return TrackSizing::MaxContent;
        }

        if value.ends_with("fr") {
            if let Ok(fr) = value.trim_end_matches("fr").parse::<f32>() {
                return TrackSizing::Flex(fr);
            }
        }

        if value.ends_with("px") {
            if let Ok(px) = value.trim_end_matches("px").parse::<f32>() {
                return TrackSizing::Fixed(px);
            }
        }

        if value.ends_with('%') {
            if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
                return TrackSizing::Percentage(pct);
            }
        }

        if value.starts_with("minmax(") && value.ends_with(')') {
            let inner = &value[7..value.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let min = Self::parse_track_sizing(parts[0], available);
                let max = Self::parse_track_sizing(parts[1], available);
                return TrackSizing::MinMax(Box::new(min), Box::new(max));
            }
        }

        if value.starts_with("fit-content(") && value.ends_with(')') {
            let inner = &value[12..value.len() - 1];
            if let Ok(max) = inner.trim_end_matches("px").parse::<f32>() {
                return TrackSizing::FitContent(max);
            }
        }

        TrackSizing::Auto
    }

    /// Resolve gap value.
    fn resolve_gap(gap: &LengthPercentageAuto, available: f32) -> f32 {
        match gap {
            LengthPercentageAuto::Length(l) => *l,
            LengthPercentageAuto::Percentage(p) => available * p / 100.0,
            LengthPercentageAuto::Auto => 0.0,
        }
    }

    /// Layout grid children.
    pub fn layout(&mut self, tree: &mut LayoutTree, container_id: LayoutBoxId) {
        // Step 1: Collect grid items
        let mut items = self.collect_grid_items(tree, container_id);

        if items.is_empty() {
            return;
        }

        // Step 2: Resolve grid placement
        self.resolve_placements(&mut items);

        // Step 3: Ensure we have enough tracks
        self.ensure_tracks(&items);

        // Step 4: Size tracks
        self.size_tracks(&items, tree);

        // Step 5: Position tracks
        self.position_tracks();

        // Step 6: Position items
        self.position_items(tree, &items);

        // Step 7: Update container size
        self.update_container_size(tree, container_id);
    }

    /// Collect grid items from children.
    fn collect_grid_items(&self, tree: &LayoutTree, container_id: LayoutBoxId) -> Vec<GridItem> {
        let children: Vec<LayoutBoxId> = tree.children(container_id).collect();
        let mut items = Vec::with_capacity(children.len());

        for child_id in children {
            let layout_box = match tree.get(child_id) {
                Some(b) => b,
                None => continue,
            };

            let style = &layout_box.style;

            // Parse grid placement properties
            let column_start = style.grid_column_start;
            let column_end = style.grid_column_end;
            let row_start = style.grid_row_start;
            let row_end = style.grid_row_end;

            // Get content size
            let content_width = layout_box.dimensions.content.width;
            let content_height = layout_box.dimensions.content.height;

            items.push(GridItem {
                box_id: child_id,
                column_start,
                column_end,
                row_start,
                row_end,
                resolved_column: (0, 1),
                resolved_row: (0, 1),
                content_width,
                content_height,
            });
        }

        items
    }

    /// Resolve grid item placements.
    fn resolve_placements(&mut self, items: &mut [GridItem]) {
        let explicit_cols = self.columns.len().max(1);
        let explicit_rows = self.rows.len().max(1);

        // Create occupancy grid
        let mut occupancy = OccupancyGrid::new(explicit_cols, explicit_rows);

        // First pass: place items with explicit positions
        for item in items.iter_mut() {
            if item.column_start > 0 && item.row_start > 0 {
                let col_start = (item.column_start - 1) as usize;
                let row_start = (item.row_start - 1) as usize;

                let col_span = if item.column_end > 0 {
                    ((item.column_end - 1) as usize).saturating_sub(col_start).max(1)
                } else if item.column_end < 0 {
                    (-item.column_end) as usize
                } else {
                    1
                };

                let row_span = if item.row_end > 0 {
                    ((item.row_end - 1) as usize).saturating_sub(row_start).max(1)
                } else if item.row_end < 0 {
                    (-item.row_end) as usize
                } else {
                    1
                };

                // Expand grid if needed
                if col_start + col_span > occupancy.columns {
                    occupancy.expand_columns(col_start + col_span);
                }
                if row_start + row_span > occupancy.rows {
                    occupancy.expand_rows(row_start + row_span);
                }

                item.resolved_column = (col_start, col_start + col_span);
                item.resolved_row = (row_start, row_start + row_span);

                occupancy.occupy_range(col_start, col_start + col_span, row_start, row_start + row_span);
            }
        }

        // Second pass: place items with auto placement
        let mut cursor_col = 0usize;
        let mut cursor_row = 0usize;

        let is_row_flow = matches!(self.auto_flow, GridAutoFlow::Row | GridAutoFlow::RowDense);

        for item in items.iter_mut() {
            if item.column_start > 0 && item.row_start > 0 {
                continue; // Already placed
            }

            let col_span = if item.column_end < 0 {
                (-item.column_end) as usize
            } else {
                1
            };

            let row_span = if item.row_end < 0 {
                (-item.row_end) as usize
            } else {
                1
            };

            // Find a place for this item
            loop {
                // Check if we can place here
                if cursor_col + col_span <= occupancy.columns
                    && occupancy.can_place(cursor_col, cursor_row, col_span, row_span)
                {
                    item.resolved_column = (cursor_col, cursor_col + col_span);
                    item.resolved_row = (cursor_row, cursor_row + row_span);
                    occupancy.occupy_range(
                        cursor_col,
                        cursor_col + col_span,
                        cursor_row,
                        cursor_row + row_span,
                    );
                    break;
                }

                // Move cursor
                if is_row_flow {
                    cursor_col += 1;
                    if cursor_col >= occupancy.columns {
                        cursor_col = 0;
                        cursor_row += 1;
                        if cursor_row >= occupancy.rows {
                            occupancy.expand_rows(cursor_row + row_span);
                        }
                    }
                } else {
                    cursor_row += 1;
                    if cursor_row >= occupancy.rows {
                        cursor_row = 0;
                        cursor_col += 1;
                        if cursor_col >= occupancy.columns {
                            occupancy.expand_columns(cursor_col + col_span);
                        }
                    }
                }
            }
        }
    }

    /// Ensure we have enough tracks for all items.
    fn ensure_tracks(&mut self, items: &[GridItem]) {
        let max_col = items.iter().map(|i| i.resolved_column.1).max().unwrap_or(1);
        let max_row = items.iter().map(|i| i.resolved_row.1).max().unwrap_or(1);

        // Add implicit column tracks
        while self.columns.len() < max_col {
            self.columns.push(GridTrack {
                sizing: TrackSizing::Auto,
                base_size: 0.0,
                growth_limit: f32::INFINITY,
                final_size: 0.0,
                start: 0.0,
            });
        }

        // Add implicit row tracks
        while self.rows.len() < max_row {
            self.rows.push(GridTrack {
                sizing: TrackSizing::Auto,
                base_size: 0.0,
                growth_limit: f32::INFINITY,
                final_size: 0.0,
                start: 0.0,
            });
        }
    }

    /// Size the tracks.
    fn size_tracks(&mut self, items: &[GridItem], tree: &LayoutTree) {
        // Calculate column sizes
        self.size_track_list(&mut self.columns, self.container_width, self.column_gap, items, true, tree);

        // Calculate row sizes
        let row_available = self.container_height.unwrap_or_else(|| {
            // Calculate intrinsic height from items
            items
                .iter()
                .filter_map(|i| tree.get(i.box_id))
                .map(|b| b.dimensions.content.height)
                .fold(0.0f32, |a, b| a.max(b))
        });

        self.size_track_list(&mut self.rows, row_available, self.row_gap, items, false, tree);
    }

    /// Size a list of tracks.
    fn size_track_list(
        &self,
        tracks: &mut Vec<GridTrack>,
        available: f32,
        gap: f32,
        items: &[GridItem],
        is_columns: bool,
        tree: &LayoutTree,
    ) {
        if tracks.is_empty() {
            return;
        }

        let total_gap = gap * (tracks.len() - 1).max(0) as f32;
        let available_for_tracks = (available - total_gap).max(0.0);

        // First pass: resolve fixed and percentage tracks
        let mut remaining = available_for_tracks;
        let mut flex_total = 0.0f32;

        for track in tracks.iter_mut() {
            match &track.sizing {
                TrackSizing::Fixed(px) => {
                    track.base_size = *px;
                    track.final_size = *px;
                    remaining -= px;
                }
                TrackSizing::Percentage(pct) => {
                    let size = available * pct / 100.0;
                    track.base_size = size;
                    track.final_size = size;
                    remaining -= size;
                }
                TrackSizing::Flex(fr) => {
                    flex_total += fr;
                }
                TrackSizing::Auto | TrackSizing::MinContent | TrackSizing::MaxContent => {
                    // Calculate from content
                    let content_size = self.calculate_content_size(tracks.len(), is_columns, items, tree);
                    track.base_size = content_size;
                }
                TrackSizing::FitContent(max) => {
                    let content_size = self.calculate_content_size(tracks.len(), is_columns, items, tree);
                    track.base_size = content_size.min(*max);
                }
                TrackSizing::MinMax(min, max) => {
                    // For now, use a simplified approach
                    track.base_size = self.resolve_track_value(min, available);
                    track.growth_limit = self.resolve_track_value(max, available);
                }
            }
        }

        // Second pass: distribute remaining space to flex tracks
        if flex_total > 0.0 && remaining > 0.0 {
            let flex_unit = remaining / flex_total;

            for track in tracks.iter_mut() {
                if let TrackSizing::Flex(fr) = &track.sizing {
                    track.final_size = flex_unit * fr;
                }
            }
        }

        // Handle auto tracks
        for track in tracks.iter_mut() {
            if matches!(track.sizing, TrackSizing::Auto | TrackSizing::MinContent | TrackSizing::MaxContent | TrackSizing::FitContent(_)) {
                if track.final_size == 0.0 {
                    track.final_size = track.base_size;
                }
            }
        }

        // Ensure minimum sizes
        for track in tracks.iter_mut() {
            track.final_size = track.final_size.max(track.base_size);
        }
    }

    /// Calculate content size for auto tracks.
    fn calculate_content_size(
        &self,
        track_count: usize,
        is_columns: bool,
        items: &[GridItem],
        tree: &LayoutTree,
    ) -> f32 {
        // Simplified: return average item size
        let sizes: Vec<f32> = items
            .iter()
            .filter_map(|i| tree.get(i.box_id))
            .map(|b| {
                if is_columns {
                    b.dimensions.content.width
                } else {
                    b.dimensions.content.height
                }
            })
            .collect();

        if sizes.is_empty() {
            return 100.0; // Default
        }

        sizes.iter().sum::<f32>() / track_count as f32
    }

    /// Resolve a track sizing value to pixels.
    fn resolve_track_value(&self, sizing: &TrackSizing, available: f32) -> f32 {
        match sizing {
            TrackSizing::Fixed(px) => *px,
            TrackSizing::Percentage(pct) => available * pct / 100.0,
            TrackSizing::Flex(_) => 0.0, // Flex is handled separately
            TrackSizing::Auto => 0.0,
            TrackSizing::MinContent => 0.0,
            TrackSizing::MaxContent => f32::INFINITY,
            TrackSizing::FitContent(max) => *max,
            TrackSizing::MinMax(min, _) => self.resolve_track_value(min, available),
        }
    }

    /// Position tracks (calculate start positions).
    fn position_tracks(&mut self) {
        // Position columns
        let mut x = 0.0;
        for (i, track) in self.columns.iter_mut().enumerate() {
            track.start = x;
            x += track.final_size;
            if i < self.columns.len() - 1 {
                x += self.column_gap;
            }
        }

        // Position rows
        let mut y = 0.0;
        for (i, track) in self.rows.iter_mut().enumerate() {
            track.start = y;
            y += track.final_size;
            if i < self.rows.len() - 1 {
                y += self.row_gap;
            }
        }
    }

    /// Position grid items.
    fn position_items(&self, tree: &mut LayoutTree, items: &[GridItem]) {
        for item in items {
            let col_start = item.resolved_column.0;
            let col_end = item.resolved_column.1;
            let row_start = item.resolved_row.0;
            let row_end = item.resolved_row.1;

            // Calculate position and size
            let x = self.columns.get(col_start).map(|t| t.start).unwrap_or(0.0);
            let y = self.rows.get(row_start).map(|t| t.start).unwrap_or(0.0);

            let width: f32 = self.columns[col_start..col_end]
                .iter()
                .map(|t| t.final_size)
                .sum::<f32>()
                + self.column_gap * (col_end - col_start).saturating_sub(1) as f32;

            let height: f32 = self.rows[row_start..row_end]
                .iter()
                .map(|t| t.final_size)
                .sum::<f32>()
                + self.row_gap * (row_end - row_start).saturating_sub(1) as f32;

            // Update layout box
            if let Some(layout_box) = tree.get_mut(item.box_id) {
                layout_box.dimensions.content.x = x;
                layout_box.dimensions.content.y = y;
                layout_box.dimensions.content.width = width;
                layout_box.dimensions.content.height = height;

                // Store grid data
                layout_box.grid_data = Some(GridLayoutData {
                    column_start: col_start,
                    column_end: col_end,
                    row_start: row_start,
                    row_end: row_end,
                });
            }
        }
    }

    /// Update container size.
    fn update_container_size(&self, tree: &mut LayoutTree, container_id: LayoutBoxId) {
        let total_width: f32 = self.columns.iter().map(|t| t.final_size).sum::<f32>()
            + self.column_gap * self.columns.len().saturating_sub(1) as f32;

        let total_height: f32 = self.rows.iter().map(|t| t.final_size).sum::<f32>()
            + self.row_gap * self.rows.len().saturating_sub(1) as f32;

        if let Some(container) = tree.get_mut(container_id) {
            if container.dimensions.content.width == 0.0 {
                container.dimensions.content.width = total_width;
            }
            if container.dimensions.content.height == 0.0 {
                container.dimensions.content.height = total_height;
            }
        }
    }
}

/// Layout a grid container.
pub fn layout_grid_container(
    tree: &mut LayoutTree,
    container_id: LayoutBoxId,
    containing_block: &ContainingBlock,
) {
    let container = match tree.get(container_id) {
        Some(c) => c,
        None => return,
    };

    let style = container.style.clone();

    let mut gfc = GridFormattingContext::new(&style, containing_block.width, containing_block.height);
    gfc.layout(tree, container_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occupancy_grid() {
        let mut grid = OccupancyGrid::new(3, 3);
        assert!(!grid.is_occupied(0, 0));

        grid.occupy(0, 0);
        assert!(grid.is_occupied(0, 0));

        assert!(grid.can_place(1, 0, 2, 1));
        assert!(!grid.can_place(0, 0, 2, 1));
    }

    #[test]
    fn test_track_sizing_parse() {
        let sizing = GridFormattingContext::parse_track_sizing("100px", 800.0);
        assert!(matches!(sizing, TrackSizing::Fixed(100.0)));

        let sizing = GridFormattingContext::parse_track_sizing("1fr", 800.0);
        assert!(matches!(sizing, TrackSizing::Flex(1.0)));

        let sizing = GridFormattingContext::parse_track_sizing("50%", 800.0);
        assert!(matches!(sizing, TrackSizing::Percentage(50.0)));
    }
}
