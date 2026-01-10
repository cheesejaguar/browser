//! Style resolver for full document styling.

use crate::computed::ComputedStyle;
use crate::stylist::Stylist;
use css_parser::media::MediaContext;
use css_parser::stylesheet::Stylesheet;
use dom::document::Document;
use dom::node::{NodeId, NodeType};
use dom::tree::DomTree;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Resolved styles for all elements in a document.
pub struct StyleResolver {
    stylist: Stylist,
    styles: RwLock<HashMap<NodeId, Arc<ComputedStyle>>>,
}

impl StyleResolver {
    pub fn new() -> Self {
        Self {
            stylist: Stylist::new(),
            styles: RwLock::new(HashMap::new()),
        }
    }

    /// Add default browser styles.
    pub fn add_default_styles(&mut self) {
        let default_css = include_str!("default.css");
        let stylesheet = css_parser::parse_css(
            default_css,
            url::Url::parse("about:blank").unwrap(),
        );
        self.stylist.add_ua_stylesheet(stylesheet);
    }

    /// Add author stylesheet.
    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.stylist.add_author_stylesheet(stylesheet);
    }

    /// Set media context.
    pub fn set_media_context(&mut self, context: MediaContext) {
        self.stylist.set_media_context(context);
    }

    /// Resolve styles for entire document.
    pub fn resolve_document(&mut self, document: &Document) {
        self.styles.write().clear();

        if let Some(root) = document.tree.root() {
            self.resolve_subtree(&document.tree, root, None);
        }
    }

    /// Resolve styles for a subtree.
    fn resolve_subtree(
        &mut self,
        tree: &DomTree,
        node_id: NodeId,
        parent_style: Option<&ComputedStyle>,
    ) {
        let node = match tree.get(node_id) {
            Some(n) => n,
            None => return,
        };

        let style = if node.node_type == NodeType::Element {
            let computed = self.stylist.compute_style(tree, node_id, parent_style);
            let style = Arc::new(computed);
            self.styles.write().insert(node_id, style.clone());
            Some(style)
        } else {
            None
        };

        // Process children
        let children: Vec<NodeId> = node.children.iter().copied().collect();
        for child in children {
            self.resolve_subtree(
                tree,
                child,
                style.as_ref().map(|s| s.as_ref()).or(parent_style),
            );
        }
    }

    /// Get computed style for a node.
    pub fn get_style(&self, node_id: NodeId) -> Option<Arc<ComputedStyle>> {
        self.styles.read().get(&node_id).cloned()
    }

    /// Invalidate styles for a subtree.
    pub fn invalidate_subtree(&mut self, tree: &DomTree, node_id: NodeId) {
        let mut to_remove = vec![node_id];

        // Collect all descendants
        let mut i = 0;
        while i < to_remove.len() {
            if let Some(node) = tree.get(to_remove[i]) {
                to_remove.extend(node.children.iter().copied());
            }
            i += 1;
        }

        // Remove from cache
        let mut styles = self.styles.write();
        for id in to_remove {
            styles.remove(&id);
        }
    }

    /// Restyle a single element.
    pub fn restyle_element(&mut self, tree: &DomTree, node_id: NodeId) {
        let parent_style = tree
            .parent(node_id)
            .and_then(|p| self.styles.read().get(&p).cloned());

        let computed = self.stylist.compute_style(
            tree,
            node_id,
            parent_style.as_ref().map(|s| s.as_ref()),
        );

        self.styles.write().insert(node_id, Arc::new(computed));
    }

    /// Check if node has style.
    pub fn has_style(&self, node_id: NodeId) -> bool {
        self.styles.read().contains_key(&node_id)
    }

    /// Get all styled nodes.
    pub fn styled_nodes(&self) -> Vec<NodeId> {
        self.styles.read().keys().copied().collect()
    }

    /// Clear all styles.
    pub fn clear(&mut self) {
        self.styles.write().clear();
        self.stylist.invalidate_cache();
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Style change detection.
#[derive(Clone, Debug, Default)]
pub struct StyleDiff {
    pub display_changed: bool,
    pub position_changed: bool,
    pub size_changed: bool,
    pub margin_changed: bool,
    pub padding_changed: bool,
    pub border_changed: bool,
    pub background_changed: bool,
    pub text_changed: bool,
    pub transform_changed: bool,
    pub opacity_changed: bool,
    pub visibility_changed: bool,
}

impl StyleDiff {
    pub fn compare(old: &ComputedStyle, new: &ComputedStyle) -> Self {
        Self {
            display_changed: old.display != new.display,
            position_changed: old.position != new.position,
            size_changed: old.width != new.width
                || old.height != new.height
                || old.min_width != new.min_width
                || old.min_height != new.min_height
                || old.max_width != new.max_width
                || old.max_height != new.max_height,
            margin_changed: old.margin != new.margin,
            padding_changed: old.padding != new.padding,
            border_changed: old.border_width != new.border_width
                || old.border_style != new.border_style
                || old.border_color != new.border_color,
            background_changed: old.background_color != new.background_color,
            text_changed: old.color != new.color
                || old.font_size != new.font_size
                || old.font_family != new.font_family
                || old.font_weight != new.font_weight,
            transform_changed: old.transform != new.transform,
            opacity_changed: (old.opacity - new.opacity).abs() > f32::EPSILON,
            visibility_changed: old.visibility != new.visibility,
        }
    }

    pub fn needs_layout(&self) -> bool {
        self.display_changed
            || self.position_changed
            || self.size_changed
            || self.margin_changed
            || self.padding_changed
            || self.border_changed
            || self.text_changed
    }

    pub fn needs_paint(&self) -> bool {
        self.background_changed
            || self.text_changed
            || self.opacity_changed
            || self.visibility_changed
    }

    pub fn needs_composite(&self) -> bool {
        self.transform_changed || self.opacity_changed
    }

    pub fn has_changes(&self) -> bool {
        self.display_changed
            || self.position_changed
            || self.size_changed
            || self.margin_changed
            || self.padding_changed
            || self.border_changed
            || self.background_changed
            || self.text_changed
            || self.transform_changed
            || self.opacity_changed
            || self.visibility_changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = StyleResolver::new();
        assert!(resolver.styled_nodes().is_empty());
    }

    #[test]
    fn test_style_diff() {
        let old = ComputedStyle::default_style();
        let mut new = ComputedStyle::default_style();
        new.opacity = 0.5;

        let diff = StyleDiff::compare(&old, &new);
        assert!(diff.opacity_changed);
        assert!(!diff.display_changed);
        assert!(diff.needs_composite());
    }
}
