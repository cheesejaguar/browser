//! Compositing layers.

use common::geometry::{Rect, Transform};
use render::display_list::{BlendMode, DisplayList};
use slotmap::{new_key_type, SlotMap};
use smallvec::SmallVec;

new_key_type! {
    /// Unique identifier for a layer.
    pub struct LayerId;
}

/// A compositing layer.
#[derive(Clone, Debug)]
pub struct Layer {
    /// Layer ID.
    pub id: LayerId,
    /// Parent layer.
    pub parent: Option<LayerId>,
    /// Child layers.
    pub children: SmallVec<[LayerId; 4]>,
    /// Layer bounds.
    pub bounds: Rect,
    /// Transform relative to parent.
    pub transform: Transform,
    /// Opacity (0.0 - 1.0).
    pub opacity: f32,
    /// Blend mode.
    pub blend_mode: BlendMode,
    /// Whether the layer content is visible.
    pub visible: bool,
    /// Whether this layer needs its own texture.
    pub needs_isolation: bool,
    /// Display list for this layer's content.
    pub display_list: DisplayList,
    /// Whether the layer content is dirty.
    pub dirty: bool,
    /// Cached texture ID (if any).
    pub texture_id: Option<u64>,
    /// Scroll offset.
    pub scroll_offset: (f32, f32),
    /// Clip to bounds.
    pub clips_to_bounds: bool,
    /// Filter effects.
    pub filters: Vec<FilterEffect>,
    /// Mask layer.
    pub mask: Option<LayerId>,
}

impl Layer {
    pub fn new(id: LayerId) -> Self {
        Self {
            id,
            parent: None,
            children: SmallVec::new(),
            bounds: Rect::default(),
            transform: Transform::identity(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
            needs_isolation: false,
            display_list: DisplayList::new(),
            dirty: true,
            texture_id: None,
            scroll_offset: (0.0, 0.0),
            clips_to_bounds: false,
            filters: Vec::new(),
            mask: None,
        }
    }

    /// Check if this layer requires compositing.
    pub fn requires_compositing(&self) -> bool {
        self.opacity < 1.0
            || !matches!(self.blend_mode, BlendMode::Normal)
            || !self.transform.is_identity()
            || !self.filters.is_empty()
            || self.mask.is_some()
            || self.needs_isolation
    }

    /// Get the accumulated transform.
    pub fn accumulated_transform(&self, tree: &LayerTree) -> Transform {
        let mut transform = self.transform.clone();

        if let Some(parent_id) = self.parent {
            if let Some(parent) = tree.get(parent_id) {
                let parent_transform = parent.accumulated_transform(tree);
                transform = parent_transform.multiply(&transform);
            }
        }

        transform
    }

    /// Get the accumulated opacity.
    pub fn accumulated_opacity(&self, tree: &LayerTree) -> f32 {
        let mut opacity = self.opacity;

        if let Some(parent_id) = self.parent {
            if let Some(parent) = tree.get(parent_id) {
                opacity *= parent.accumulated_opacity(tree);
            }
        }

        opacity
    }

    /// Mark the layer as dirty.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

/// Filter effect.
#[derive(Clone, Debug)]
pub enum FilterEffect {
    /// Blur filter.
    Blur { radius: f32 },
    /// Brightness adjustment.
    Brightness { amount: f32 },
    /// Contrast adjustment.
    Contrast { amount: f32 },
    /// Grayscale conversion.
    Grayscale { amount: f32 },
    /// Hue rotation.
    HueRotate { angle: f32 },
    /// Invert colors.
    Invert { amount: f32 },
    /// Opacity.
    Opacity { amount: f32 },
    /// Saturate.
    Saturate { amount: f32 },
    /// Sepia.
    Sepia { amount: f32 },
    /// Drop shadow.
    DropShadow {
        offset_x: f32,
        offset_y: f32,
        blur: f32,
        color: common::color::Color,
    },
}

/// Layer tree.
pub struct LayerTree {
    layers: SlotMap<LayerId, Layer>,
    root: Option<LayerId>,
}

impl LayerTree {
    pub fn new() -> Self {
        Self {
            layers: SlotMap::with_key(),
            root: None,
        }
    }

    /// Create a new layer.
    pub fn create_layer(&mut self) -> LayerId {
        self.layers.insert_with_key(|id| Layer::new(id))
    }

    /// Get the root layer.
    pub fn root(&self) -> Option<LayerId> {
        self.root
    }

    /// Set the root layer.
    pub fn set_root(&mut self, layer_id: LayerId) {
        self.root = Some(layer_id);
    }

    /// Get a layer by ID.
    pub fn get(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(id)
    }

    /// Get a mutable layer by ID.
    pub fn get_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.get_mut(id)
    }

    /// Append a child layer.
    pub fn append_child(&mut self, parent_id: LayerId, child_id: LayerId) {
        if let Some(child) = self.layers.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
        if let Some(parent) = self.layers.get_mut(parent_id) {
            parent.children.push(child_id);
        }
    }

    /// Remove a layer.
    pub fn remove(&mut self, layer_id: LayerId) {
        // Remove from parent
        if let Some(parent_id) = self.layers.get(layer_id).and_then(|l| l.parent) {
            if let Some(parent) = self.layers.get_mut(parent_id) {
                parent.children.retain(|&id| id != layer_id);
            }
        }

        // Remove layer and children
        let mut to_remove = vec![layer_id];
        let mut i = 0;
        while i < to_remove.len() {
            if let Some(layer) = self.layers.get(to_remove[i]) {
                to_remove.extend(layer.children.iter().copied());
            }
            i += 1;
        }

        for id in to_remove {
            self.layers.remove(id);
        }

        if self.root == Some(layer_id) {
            self.root = None;
        }
    }

    /// Iterate over all layers.
    pub fn iter(&self) -> impl Iterator<Item = (LayerId, &Layer)> {
        self.layers.iter()
    }

    /// Get all layers that need compositing.
    pub fn compositing_layers(&self) -> Vec<LayerId> {
        self.layers
            .iter()
            .filter(|(_, layer)| layer.requires_compositing())
            .map(|(id, _)| id)
            .collect()
    }

    /// Mark a subtree as dirty.
    pub fn mark_subtree_dirty(&mut self, layer_id: LayerId) {
        let mut stack = vec![layer_id];

        while let Some(id) = stack.pop() {
            if let Some(layer) = self.layers.get_mut(id) {
                layer.dirty = true;
                stack.extend(layer.children.iter().copied());
            }
        }
    }

    /// Get layers in paint order.
    pub fn paint_order(&self) -> Vec<LayerId> {
        let mut result = Vec::new();

        if let Some(root_id) = self.root {
            self.collect_paint_order(root_id, &mut result);
        }

        result
    }

    fn collect_paint_order(&self, layer_id: LayerId, result: &mut Vec<LayerId>) {
        if let Some(layer) = self.layers.get(layer_id) {
            if !layer.visible {
                return;
            }

            // Children first (back to front)
            for &child_id in &layer.children {
                self.collect_paint_order(child_id, result);
            }

            result.push(layer_id);
        }
    }

    /// Clear the tree.
    pub fn clear(&mut self) {
        self.layers.clear();
        self.root = None;
    }

    /// Get number of layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Check if tree is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}

impl Default for LayerTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_tree() {
        let mut tree = LayerTree::new();

        let root = tree.create_layer();
        tree.set_root(root);

        let child1 = tree.create_layer();
        let child2 = tree.create_layer();

        tree.append_child(root, child1);
        tree.append_child(root, child2);

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.get(root).unwrap().children.len(), 2);
    }

    #[test]
    fn test_compositing_check() {
        let mut tree = LayerTree::new();
        let layer_id = tree.create_layer();

        // Default layer doesn't need compositing
        assert!(!tree.get(layer_id).unwrap().requires_compositing());

        // Layer with opacity needs compositing
        tree.get_mut(layer_id).unwrap().opacity = 0.5;
        assert!(tree.get(layer_id).unwrap().requires_compositing());
    }
}
