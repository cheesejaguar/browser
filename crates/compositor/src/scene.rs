//! Scene graph for compositing.

use crate::layer::{Layer, LayerId, LayerTree};
use common::geometry::{Point, Rect, Transform};
use parking_lot::RwLock;
use render::display_list::DisplayList;
use std::collections::HashMap;

/// A compositing scene.
pub struct Scene {
    /// Layer tree.
    layer_tree: LayerTree,
    /// Scene bounds.
    bounds: Rect,
    /// Background color.
    background: common::color::Color,
    /// Scroll position.
    scroll_position: Point,
    /// Dirty regions needing repaint.
    dirty_regions: Vec<Rect>,
}

impl Scene {
    pub fn new(width: f32, height: f32) -> Self {
        let mut tree = LayerTree::new();
        let root = tree.create_layer();
        tree.set_root(root);

        if let Some(layer) = tree.get_mut(root) {
            layer.bounds = Rect::new(0.0, 0.0, width, height);
        }

        Self {
            layer_tree: tree,
            bounds: Rect::new(0.0, 0.0, width, height),
            background: common::color::Color::white(),
            scroll_position: Point::new(0.0, 0.0),
            dirty_regions: Vec::new(),
        }
    }

    /// Get the layer tree.
    pub fn layer_tree(&self) -> &LayerTree {
        &self.layer_tree
    }

    /// Get mutable layer tree.
    pub fn layer_tree_mut(&mut self) -> &mut LayerTree {
        &mut self.layer_tree
    }

    /// Get root layer.
    pub fn root_layer(&self) -> Option<LayerId> {
        self.layer_tree.root()
    }

    /// Create a new layer.
    pub fn create_layer(&mut self) -> LayerId {
        self.layer_tree.create_layer()
    }

    /// Get a layer.
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layer_tree.get(id)
    }

    /// Get a mutable layer.
    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layer_tree.get_mut(id)
    }

    /// Set the display list for a layer.
    pub fn set_layer_content(&mut self, layer_id: LayerId, display_list: DisplayList) {
        if let Some(layer) = self.layer_tree.get_mut(layer_id) {
            layer.display_list = display_list;
            layer.dirty = true;
        }
    }

    /// Resize the scene.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;

        if let Some(root_id) = self.layer_tree.root() {
            if let Some(root) = self.layer_tree.get_mut(root_id) {
                root.bounds.width = width;
                root.bounds.height = height;
                root.dirty = true;
            }
        }
    }

    /// Get scene bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds.clone()
    }

    /// Set background color.
    pub fn set_background(&mut self, color: common::color::Color) {
        self.background = color;
    }

    /// Get background color.
    pub fn background(&self) -> common::color::Color {
        self.background.clone()
    }

    /// Set scroll position.
    pub fn set_scroll_position(&mut self, x: f32, y: f32) {
        self.scroll_position = Point::new(x, y);
    }

    /// Get scroll position.
    pub fn scroll_position(&self) -> Point {
        self.scroll_position.clone()
    }

    /// Add a dirty region.
    pub fn add_dirty_region(&mut self, rect: Rect) {
        self.dirty_regions.push(rect);
    }

    /// Get dirty regions.
    pub fn dirty_regions(&self) -> &[Rect] {
        &self.dirty_regions
    }

    /// Clear dirty regions.
    pub fn clear_dirty_regions(&mut self) {
        self.dirty_regions.clear();
    }

    /// Mark the entire scene as dirty.
    pub fn mark_all_dirty(&mut self) {
        self.dirty_regions.clear();
        self.dirty_regions.push(self.bounds.clone());

        if let Some(root_id) = self.layer_tree.root() {
            self.layer_tree.mark_subtree_dirty(root_id);
        }
    }

    /// Hit test to find the layer at a point.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<LayerId> {
        let point = Point::new(x, y);

        // Test layers in reverse paint order (front to back)
        let paint_order = self.layer_tree.paint_order();

        for layer_id in paint_order.into_iter().rev() {
            if let Some(layer) = self.layer_tree.get(layer_id) {
                if layer.visible && layer.bounds.contains_point(point.clone()) {
                    return Some(layer_id);
                }
            }
        }

        None
    }

    /// Get all layers that need compositing.
    pub fn compositing_layers(&self) -> Vec<LayerId> {
        self.layer_tree.compositing_layers()
    }
}

/// Scene builder for constructing scenes from layout.
pub struct SceneBuilder {
    scene: Scene,
    current_layer: Option<LayerId>,
}

impl SceneBuilder {
    pub fn new(width: f32, height: f32) -> Self {
        let scene = Scene::new(width, height);
        let current_layer = scene.root_layer();

        Self {
            scene,
            current_layer,
        }
    }

    /// Push a new layer.
    pub fn push_layer(&mut self, bounds: Rect) -> LayerId {
        let layer_id = self.scene.create_layer();

        if let Some(layer) = self.scene.get_layer_mut(layer_id) {
            layer.bounds = bounds;
        }

        if let Some(parent_id) = self.current_layer {
            self.scene.layer_tree_mut().append_child(parent_id, layer_id);
        }

        self.current_layer = Some(layer_id);
        layer_id
    }

    /// Pop the current layer.
    pub fn pop_layer(&mut self) {
        if let Some(layer_id) = self.current_layer {
            if let Some(layer) = self.scene.get_layer(layer_id) {
                self.current_layer = layer.parent;
            }
        }
    }

    /// Set layer opacity.
    pub fn set_opacity(&mut self, opacity: f32) {
        if let Some(layer_id) = self.current_layer {
            if let Some(layer) = self.scene.get_layer_mut(layer_id) {
                layer.opacity = opacity;
            }
        }
    }

    /// Set layer transform.
    pub fn set_transform(&mut self, transform: Transform) {
        if let Some(layer_id) = self.current_layer {
            if let Some(layer) = self.scene.get_layer_mut(layer_id) {
                layer.transform = transform;
            }
        }
    }

    /// Set layer display list.
    pub fn set_display_list(&mut self, display_list: DisplayList) {
        if let Some(layer_id) = self.current_layer {
            self.scene.set_layer_content(layer_id, display_list);
        }
    }

    /// Build the scene.
    pub fn build(self) -> Scene {
        self.scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new(800.0, 600.0);
        assert!(scene.root_layer().is_some());
        assert_eq!(scene.bounds().width, 800.0);
        assert_eq!(scene.bounds().height, 600.0);
    }

    #[test]
    fn test_scene_builder() {
        let mut builder = SceneBuilder::new(800.0, 600.0);

        let layer1 = builder.push_layer(Rect::new(0.0, 0.0, 400.0, 300.0));
        builder.set_opacity(0.8);
        builder.pop_layer();

        let scene = builder.build();
        assert_eq!(scene.layer_tree().len(), 2); // root + layer1
    }
}
