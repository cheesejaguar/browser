//! Flexbox layout implementation (CSS Flexible Box Layout Module).

use crate::box_model::{BoxDimensions, ContainingBlock};
use crate::layout_box::{FlexLayoutData, LayoutBox, LayoutBoxId};
use crate::tree::LayoutTree;
use common::geometry::EdgeSizes;
use style::computed::{
    AlignContent, AlignItems, AlignSelf, ComputedStyle, FlexDirection, FlexWrap,
    JustifyContent, LengthPercentage, LengthPercentageAuto,
};

/// Flexbox formatting context.
pub struct FlexFormattingContext {
    /// Flex direction.
    direction: FlexDirection,
    /// Flex wrap.
    wrap: FlexWrap,
    /// Justify content.
    justify_content: JustifyContent,
    /// Align items.
    align_items: AlignItems,
    /// Align content.
    align_content: AlignContent,
    /// Container main size.
    main_size: f32,
    /// Container cross size.
    cross_size: Option<f32>,
    /// Is main axis horizontal.
    is_row: bool,
    /// Is main axis reversed.
    is_reversed: bool,
    /// Is cross axis reversed (wrap-reverse).
    is_cross_reversed: bool,
}

/// Flex item for layout calculations.
#[derive(Clone, Debug)]
struct FlexItem {
    box_id: LayoutBoxId,
    /// Base size before flex calculations.
    base_size: f32,
    /// Flex grow factor.
    flex_grow: f32,
    /// Flex shrink factor.
    flex_shrink: f32,
    /// Hypothetical main size.
    hypothetical_main_size: f32,
    /// Target main size after flex calculations.
    target_main_size: f32,
    /// Cross size.
    cross_size: f32,
    /// Main axis margin.
    main_margin_start: f32,
    main_margin_end: f32,
    /// Cross axis margin.
    cross_margin_start: f32,
    cross_margin_end: f32,
    /// Is frozen (can't flex anymore).
    frozen: bool,
    /// Violation (how much item overflows/underflows).
    violation: f32,
    /// Align self value.
    align_self: AlignSelf,
    /// Order value.
    order: i32,
}

/// Flex line for multi-line flex containers.
#[derive(Clone, Debug)]
struct FlexLine {
    items: Vec<FlexItem>,
    main_size: f32,
    cross_size: f32,
}

impl FlexFormattingContext {
    pub fn new(style: &ComputedStyle, main_size: f32, cross_size: Option<f32>) -> Self {
        let direction = style.flex_direction.clone();
        let is_row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
        let is_reversed = matches!(
            direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );
        let is_cross_reversed = matches!(style.flex_wrap, FlexWrap::WrapReverse);

        Self {
            direction,
            wrap: style.flex_wrap.clone(),
            justify_content: style.justify_content.clone(),
            align_items: style.align_items.clone(),
            align_content: style.align_content.clone(),
            main_size,
            cross_size,
            is_row,
            is_reversed,
            is_cross_reversed,
        }
    }

    /// Layout flex children.
    pub fn layout(&mut self, tree: &mut LayoutTree, container_id: LayoutBoxId) {
        // Step 1: Generate flex items
        let mut items = self.generate_flex_items(tree, container_id);

        if items.is_empty() {
            return;
        }

        // Step 2: Determine main size of items
        self.determine_main_sizes(&mut items, tree);

        // Step 3: Collect items into flex lines
        let mut lines = self.collect_into_lines(items);

        // Step 4: Resolve flexible lengths
        for line in &mut lines {
            self.resolve_flexible_lengths(line);
        }

        // Step 5: Determine cross sizes
        self.determine_cross_sizes(&mut lines, tree);

        // Step 6: Align items within lines
        self.align_items_in_lines(&mut lines);

        // Step 7: Align lines (for multi-line containers)
        let total_cross_size = self.align_lines(&mut lines);

        // Step 8: Position items
        self.position_items(tree, container_id, &lines);

        // Update container dimensions
        if let Some(container) = tree.get_mut(container_id) {
            if self.is_row {
                if container.dimensions.content.height == 0.0 {
                    container.dimensions.content.height = total_cross_size;
                }
            } else {
                if container.dimensions.content.width == 0.0 {
                    container.dimensions.content.width = total_cross_size;
                }
            }
        }
    }

    /// Generate flex items from children.
    fn generate_flex_items(&self, tree: &LayoutTree, container_id: LayoutBoxId) -> Vec<FlexItem> {
        let children: Vec<LayoutBoxId> = tree.children(container_id).collect();
        let mut items = Vec::with_capacity(children.len());

        for child_id in children {
            let layout_box = match tree.get(child_id) {
                Some(b) => b,
                None => continue,
            };

            let style = &layout_box.style;

            // Get flex properties
            let flex_grow = style.flex_grow;
            let flex_shrink = style.flex_shrink;
            let flex_basis = &style.flex_basis;

            // Calculate base size
            let base_size = self.calculate_base_size(layout_box, flex_basis);

            // Get margins
            let (main_margin_start, main_margin_end, cross_margin_start, cross_margin_end) =
                self.get_margins(style);

            // Get align-self
            let align_self = style.align_self.clone();

            let item = FlexItem {
                box_id: child_id,
                base_size,
                flex_grow,
                flex_shrink,
                hypothetical_main_size: base_size,
                target_main_size: base_size,
                cross_size: 0.0,
                main_margin_start,
                main_margin_end,
                cross_margin_start,
                cross_margin_end,
                frozen: false,
                violation: 0.0,
                align_self,
                order: style.order,
            };

            items.push(item);
        }

        // Sort by order property
        items.sort_by_key(|item| item.order);

        items
    }

    /// Calculate base size from flex-basis.
    fn calculate_base_size(&self, layout_box: &LayoutBox, flex_basis: &LengthPercentageAuto) -> f32 {
        match flex_basis {
            LengthPercentageAuto::Length(l) => *l,
            LengthPercentageAuto::Percentage(p) => self.main_size * p / 100.0,
            LengthPercentageAuto::Auto => {
                // Use content size
                if self.is_row {
                    layout_box.dimensions.content.width
                } else {
                    layout_box.dimensions.content.height
                }
            }
        }
    }

    /// Get margins in main/cross axis terms.
    fn get_margins(&self, style: &ComputedStyle) -> (f32, f32, f32, f32) {
        let resolve = |v: &LengthPercentageAuto| -> f32 {
            match v {
                LengthPercentageAuto::Length(l) => *l,
                LengthPercentageAuto::Percentage(p) => self.main_size * p / 100.0,
                LengthPercentageAuto::Auto => 0.0,
            }
        };

        let margin_top = resolve(&style.margin.top);
        let margin_right = resolve(&style.margin.right);
        let margin_bottom = resolve(&style.margin.bottom);
        let margin_left = resolve(&style.margin.left);

        if self.is_row {
            if self.is_reversed {
                (margin_right, margin_left, margin_top, margin_bottom)
            } else {
                (margin_left, margin_right, margin_top, margin_bottom)
            }
        } else {
            if self.is_reversed {
                (margin_bottom, margin_top, margin_left, margin_right)
            } else {
                (margin_top, margin_bottom, margin_left, margin_right)
            }
        }
    }

    /// Determine main sizes of items.
    fn determine_main_sizes(&self, items: &mut [FlexItem], tree: &LayoutTree) {
        for item in items {
            // Apply min/max constraints
            let layout_box = match tree.get(item.box_id) {
                Some(b) => b,
                None => continue,
            };

            let (min_main, max_main) = if self.is_row {
                (
                    self.resolve_length(&layout_box.style.min_width),
                    self.resolve_max_length(&layout_box.style.max_width),
                )
            } else {
                (
                    self.resolve_length(&layout_box.style.min_height),
                    self.resolve_max_length(&layout_box.style.max_height),
                )
            };

            item.hypothetical_main_size = item.base_size.max(min_main).min(max_main);
        }
    }

    /// Resolve a LengthPercentageAuto to pixels.
    fn resolve_length(&self, value: &LengthPercentageAuto) -> f32 {
        match value {
            LengthPercentageAuto::Length(l) => *l,
            LengthPercentageAuto::Percentage(p) => self.main_size * p / 100.0,
            LengthPercentageAuto::Auto => 0.0,
        }
    }

    /// Resolve max-width/height.
    fn resolve_max_length(&self, value: &LengthPercentageAuto) -> f32 {
        match value {
            LengthPercentageAuto::Length(l) => *l,
            LengthPercentageAuto::Percentage(p) => self.main_size * p / 100.0,
            LengthPercentageAuto::Auto => f32::INFINITY,
        }
    }

    /// Collect items into flex lines.
    fn collect_into_lines(&self, items: Vec<FlexItem>) -> Vec<FlexLine> {
        if matches!(self.wrap, FlexWrap::Nowrap) {
            // Single line
            let main_size: f32 = items
                .iter()
                .map(|i| i.hypothetical_main_size + i.main_margin_start + i.main_margin_end)
                .sum();

            return vec![FlexLine {
                items,
                main_size,
                cross_size: 0.0,
            }];
        }

        // Multi-line
        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_main_size = 0.0f32;

        for item in items {
            let item_main = item.hypothetical_main_size + item.main_margin_start + item.main_margin_end;

            if !current_line.is_empty() && current_main_size + item_main > self.main_size {
                // Start new line
                lines.push(FlexLine {
                    items: std::mem::take(&mut current_line),
                    main_size: current_main_size,
                    cross_size: 0.0,
                });
                current_main_size = 0.0;
            }

            current_main_size += item_main;
            current_line.push(item);
        }

        if !current_line.is_empty() {
            lines.push(FlexLine {
                items: current_line,
                main_size: current_main_size,
                cross_size: 0.0,
            });
        }

        lines
    }

    /// Resolve flexible lengths for a line.
    fn resolve_flexible_lengths(&self, line: &mut FlexLine) {
        // Calculate free space
        let used_space: f32 = line
            .items
            .iter()
            .map(|i| i.hypothetical_main_size + i.main_margin_start + i.main_margin_end)
            .sum();

        let free_space = self.main_size - used_space;

        if free_space.abs() < 0.01 {
            // No flex needed
            for item in &mut line.items {
                item.target_main_size = item.hypothetical_main_size;
            }
            return;
        }

        // Determine if we're growing or shrinking
        let is_growing = free_space > 0.0;

        // Initialize items
        for item in &mut line.items {
            item.target_main_size = item.hypothetical_main_size;
            item.frozen = false;

            // Check if item can flex
            let flex_factor = if is_growing {
                item.flex_grow
            } else {
                item.flex_shrink
            };

            if flex_factor == 0.0 {
                item.frozen = true;
            }
        }

        // Iteratively resolve flex
        loop {
            // Calculate total flex factor for unfrozen items
            let total_flex: f32 = line
                .items
                .iter()
                .filter(|i| !i.frozen)
                .map(|i| if is_growing { i.flex_grow } else { i.flex_shrink })
                .sum();

            if total_flex == 0.0 {
                break;
            }

            // Calculate remaining free space
            let used: f32 = line
                .items
                .iter()
                .map(|i| {
                    if i.frozen {
                        i.target_main_size
                    } else {
                        i.hypothetical_main_size
                    }
                })
                .sum();

            let remaining_space = self.main_size
                - used
                - line.items.iter().map(|i| i.main_margin_start + i.main_margin_end).sum::<f32>();

            if remaining_space.abs() < 0.01 {
                break;
            }

            // Distribute space
            let mut any_frozen = false;

            for item in &mut line.items {
                if item.frozen {
                    continue;
                }

                let flex_factor = if is_growing {
                    item.flex_grow
                } else {
                    item.flex_shrink
                };

                let ratio = flex_factor / total_flex;
                let flex_amount = remaining_space * ratio;

                if is_growing {
                    item.target_main_size = item.hypothetical_main_size + flex_amount;
                } else {
                    // For shrinking, scale by flex-basis
                    let scaled_shrink = item.flex_shrink * item.base_size;
                    let total_scaled: f32 = line
                        .items
                        .iter()
                        .filter(|i| !i.frozen)
                        .map(|i| i.flex_shrink * i.base_size)
                        .sum();

                    if total_scaled > 0.0 {
                        let shrink_ratio = scaled_shrink / total_scaled;
                        item.target_main_size =
                            item.hypothetical_main_size + remaining_space * shrink_ratio;
                    }
                }

                // Check for min/max violations
                item.target_main_size = item.target_main_size.max(0.0);

                // Freeze if violated
                if item.target_main_size <= 0.0 {
                    item.target_main_size = 0.0;
                    item.frozen = true;
                    any_frozen = true;
                }
            }

            if !any_frozen {
                break;
            }
        }

        // Finalize
        for item in &mut line.items {
            if !item.frozen {
                item.frozen = true;
            }
        }
    }

    /// Determine cross sizes of items.
    fn determine_cross_sizes(&self, lines: &mut [FlexLine], tree: &LayoutTree) {
        for line in lines {
            let mut max_cross = 0.0f32;

            for item in &mut line.items {
                let layout_box = match tree.get(item.box_id) {
                    Some(b) => b,
                    None => continue,
                };

                // Get specified cross size
                let specified_cross = if self.is_row {
                    &layout_box.style.height
                } else {
                    &layout_box.style.width
                };

                item.cross_size = match specified_cross {
                    LengthPercentageAuto::Length(l) => *l,
                    LengthPercentageAuto::Percentage(p) => {
                        if let Some(cs) = self.cross_size {
                            cs * p / 100.0
                        } else {
                            // Use intrinsic size
                            if self.is_row {
                                layout_box.dimensions.content.height
                            } else {
                                layout_box.dimensions.content.width
                            }
                        }
                    }
                    LengthPercentageAuto::Auto => {
                        // Use intrinsic size or stretch
                        if self.is_row {
                            layout_box.dimensions.content.height.max(layout_box.style.font_size * 1.2)
                        } else {
                            layout_box.dimensions.content.width.max(item.target_main_size)
                        }
                    }
                };

                let item_cross = item.cross_size + item.cross_margin_start + item.cross_margin_end;
                max_cross = max_cross.max(item_cross);
            }

            line.cross_size = max_cross;
        }
    }

    /// Align items within lines.
    fn align_items_in_lines(&self, lines: &mut [FlexLine]) {
        for line in lines {
            for item in &mut line.items {
                // Determine alignment
                let alignment = match &item.align_self {
                    AlignSelf::Auto => self.align_items.clone(),
                    AlignSelf::FlexStart => AlignItems::FlexStart,
                    AlignSelf::FlexEnd => AlignItems::FlexEnd,
                    AlignSelf::Center => AlignItems::Center,
                    AlignSelf::Baseline => AlignItems::Baseline,
                    AlignSelf::Stretch => AlignItems::Stretch,
                };

                // Apply stretch if needed
                if matches!(alignment, AlignItems::Stretch) {
                    let available = line.cross_size - item.cross_margin_start - item.cross_margin_end;
                    item.cross_size = available.max(item.cross_size);
                }
            }
        }
    }

    /// Align lines within container.
    fn align_lines(&self, lines: &mut [FlexLine]) -> f32 {
        let total_cross: f32 = lines.iter().map(|l| l.cross_size).sum();

        if let Some(container_cross) = self.cross_size {
            let free_space = container_cross - total_cross;

            if free_space > 0.0 && lines.len() > 1 {
                match self.align_content {
                    AlignContent::FlexStart => {}
                    AlignContent::FlexEnd => {
                        // Add offset to first line
                    }
                    AlignContent::Center => {
                        // Add half offset to first line
                    }
                    AlignContent::SpaceBetween => {
                        // Distribute space between lines
                        if lines.len() > 1 {
                            let gap = free_space / (lines.len() - 1) as f32;
                            for (i, line) in lines.iter_mut().enumerate().skip(1) {
                                line.cross_size += gap * i as f32;
                            }
                        }
                    }
                    AlignContent::SpaceAround => {
                        let gap = free_space / lines.len() as f32;
                        for (i, line) in lines.iter_mut().enumerate() {
                            line.cross_size += gap * (2 * i + 1) as f32 / 2.0;
                        }
                    }
                    AlignContent::Stretch => {
                        let extra = free_space / lines.len() as f32;
                        for line in lines.iter_mut() {
                            line.cross_size += extra;
                        }
                    }
                }
            }

            container_cross
        } else {
            total_cross
        }
    }

    /// Position items in the layout tree.
    fn position_items(
        &self,
        tree: &mut LayoutTree,
        container_id: LayoutBoxId,
        lines: &[FlexLine],
    ) {
        let mut cross_pos = 0.0f32;

        let lines_iter: Box<dyn Iterator<Item = &FlexLine>> = if self.is_cross_reversed {
            Box::new(lines.iter().rev())
        } else {
            Box::new(lines.iter())
        };

        for line in lines_iter {
            // Calculate main axis positioning
            let used_main: f32 = line
                .items
                .iter()
                .map(|i| i.target_main_size + i.main_margin_start + i.main_margin_end)
                .sum();

            let free_space = (self.main_size - used_main).max(0.0);

            let (mut main_pos, gap) = match self.justify_content {
                JustifyContent::FlexStart => (0.0, 0.0),
                JustifyContent::FlexEnd => (free_space, 0.0),
                JustifyContent::Center => (free_space / 2.0, 0.0),
                JustifyContent::SpaceBetween => {
                    let gap = if line.items.len() > 1 {
                        free_space / (line.items.len() - 1) as f32
                    } else {
                        0.0
                    };
                    (0.0, gap)
                }
                JustifyContent::SpaceAround => {
                    let gap = free_space / line.items.len() as f32;
                    (gap / 2.0, gap)
                }
                JustifyContent::SpaceEvenly => {
                    let gap = free_space / (line.items.len() + 1) as f32;
                    (gap, gap)
                }
            };

            // Position items
            let items_iter: Box<dyn Iterator<Item = &FlexItem>> = if self.is_reversed {
                Box::new(line.items.iter().rev())
            } else {
                Box::new(line.items.iter())
            };

            for item in items_iter {
                main_pos += item.main_margin_start;

                // Calculate cross position within line
                let alignment = match &item.align_self {
                    AlignSelf::Auto => self.align_items.clone(),
                    AlignSelf::FlexStart => AlignItems::FlexStart,
                    AlignSelf::FlexEnd => AlignItems::FlexEnd,
                    AlignSelf::Center => AlignItems::Center,
                    AlignSelf::Baseline => AlignItems::Baseline,
                    AlignSelf::Stretch => AlignItems::Stretch,
                };

                let item_cross_total = item.cross_size + item.cross_margin_start + item.cross_margin_end;
                let cross_free = line.cross_size - item_cross_total;

                let item_cross_pos = cross_pos + item.cross_margin_start + match alignment {
                    AlignItems::FlexStart => 0.0,
                    AlignItems::FlexEnd => cross_free,
                    AlignItems::Center => cross_free / 2.0,
                    AlignItems::Baseline => 0.0, // Simplified
                    AlignItems::Stretch => 0.0,
                };

                // Set dimensions
                if let Some(layout_box) = tree.get_mut(item.box_id) {
                    if self.is_row {
                        layout_box.dimensions.content.x = main_pos;
                        layout_box.dimensions.content.y = item_cross_pos;
                        layout_box.dimensions.content.width = item.target_main_size;
                        layout_box.dimensions.content.height = item.cross_size;
                    } else {
                        layout_box.dimensions.content.x = item_cross_pos;
                        layout_box.dimensions.content.y = main_pos;
                        layout_box.dimensions.content.width = item.cross_size;
                        layout_box.dimensions.content.height = item.target_main_size;
                    }

                    // Store flex data
                    layout_box.flex_data = Some(FlexLayoutData {
                        main_size: item.target_main_size,
                        cross_size: item.cross_size,
                        main_position: main_pos,
                        cross_position: item_cross_pos,
                    });
                }

                main_pos += item.target_main_size + item.main_margin_end + gap;
            }

            cross_pos += line.cross_size;
        }
    }
}

/// Layout a flex container.
pub fn layout_flex_container(
    tree: &mut LayoutTree,
    container_id: LayoutBoxId,
    containing_block: &ContainingBlock,
) {
    let container = match tree.get(container_id) {
        Some(c) => c,
        None => return,
    };

    let style = container.style.clone();
    let main_size = containing_block.width;
    let cross_size = containing_block.height;

    let mut ffc = FlexFormattingContext::new(&style, main_size, cross_size);
    ffc.layout(tree, container_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_flex_context_creation() {
        let style = ComputedStyle::default_style();
        let ffc = FlexFormattingContext::new(&style, 800.0, Some(600.0));
        assert!(ffc.is_row);
        assert!(!ffc.is_reversed);
    }
}
