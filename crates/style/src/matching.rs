//! Selector matching.

use css_parser::selector::{AttributeSelector, CaseSensitivity, Combinator, Selector, SelectorList};
use css_parser::stylesheet::StyleRule;
use dom::element::ElementData;
use dom::node::{Node, NodeId};
use dom::tree::DomTree;

/// Match result with specificity.
#[derive(Clone, Debug)]
pub struct MatchResult {
    pub matches: bool,
    pub specificity: css_parser::selector::Specificity,
}

/// Match a selector list against an element.
pub fn match_selectors(
    selector_list: &SelectorList,
    element: &ElementData,
    tree: &DomTree,
    node_id: NodeId,
) -> Option<css_parser::selector::Specificity> {
    selector_list
        .selectors
        .iter()
        .filter(|s| match_selector(s, element, tree, node_id))
        .map(|s| s.specificity())
        .max()
}

/// Match a single selector against an element.
pub fn match_selector(
    selector: &Selector,
    element: &ElementData,
    tree: &DomTree,
    node_id: NodeId,
) -> bool {
    // First check the simple selector parts
    if !match_simple_selector(selector, element) {
        return false;
    }

    // Then check combinator chain
    match (&selector.combinator, &selector.next) {
        (None, _) => true,
        (Some(combinator), Some(next)) => {
            match_with_combinator(combinator, next.as_ref(), tree, node_id)
        }
        _ => true,
    }
}

/// Match simple selector (without combinators).
fn match_simple_selector(selector: &Selector, element: &ElementData) -> bool {
    // Universal matches everything
    if selector.universal && selector.tag.is_none() && selector.id.is_none()
        && selector.classes.is_empty() && selector.attributes.is_empty()
        && selector.pseudo_classes.is_empty() {
        return true;
    }

    // Check tag
    if let Some(ref tag) = selector.tag {
        if element.tag_name.as_str() != tag {
            return false;
        }
    }

    // Check ID
    if let Some(ref id) = selector.id {
        match &element.id {
            Some(elem_id) if elem_id.as_ref() == id => {}
            _ => return false,
        }
    }

    // Check classes
    for class in &selector.classes {
        if !element.has_class(class) {
            return false;
        }
    }

    // Check attributes
    for attr in &selector.attributes {
        if !match_attribute_selector(attr, element) {
            return false;
        }
    }

    // Pseudo-classes would need more context (hover state, nth-child, etc.)
    // For now, assume they match

    true
}

/// Match attribute selector.
fn match_attribute_selector(selector: &AttributeSelector, element: &ElementData) -> bool {
    let attr_value = match element.get_attribute(&selector.name) {
        Some(v) => v,
        None => return false,
    };

    let (attr_value, expected) = match selector.case_sensitivity {
        CaseSensitivity::Insensitive => (
            attr_value.to_ascii_lowercase(),
            selector.value.as_ref().map(|v| v.to_ascii_lowercase()),
        ),
        _ => (attr_value.to_string(), selector.value.clone()),
    };

    match (selector.operator.as_deref(), expected) {
        (None, _) => true,
        (Some("="), Some(v)) => attr_value == v,
        (Some("~="), Some(v)) => attr_value.split_whitespace().any(|w| w == v),
        (Some("|="), Some(v)) => attr_value == v || attr_value.starts_with(&format!("{}-", v)),
        (Some("^="), Some(v)) => attr_value.starts_with(&v),
        (Some("$="), Some(v)) => attr_value.ends_with(&v),
        (Some("*="), Some(v)) => attr_value.contains(&v),
        _ => false,
    }
}

/// Match with combinator.
fn match_with_combinator(
    combinator: &Combinator,
    next_selector: &Selector,
    tree: &DomTree,
    node_id: NodeId,
) -> bool {
    match combinator {
        Combinator::Descendant => {
            // Any ancestor must match
            let mut current = tree.parent(node_id);
            while let Some(ancestor_id) = current {
                if let Some(node) = tree.get(ancestor_id) {
                    if let Some(elem) = node.as_element() {
                        if match_selector(next_selector, elem, tree, ancestor_id) {
                            return true;
                        }
                    }
                }
                current = tree.parent(ancestor_id);
            }
            false
        }
        Combinator::Child => {
            // Direct parent must match
            if let Some(parent_id) = tree.parent(node_id) {
                if let Some(node) = tree.get(parent_id) {
                    if let Some(elem) = node.as_element() {
                        return match_selector(next_selector, elem, tree, parent_id);
                    }
                }
            }
            false
        }
        Combinator::NextSibling => {
            // Immediately preceding sibling must match
            if let Some(prev_id) = tree.prev_sibling(node_id) {
                if let Some(node) = tree.get(prev_id) {
                    if let Some(elem) = node.as_element() {
                        return match_selector(next_selector, elem, tree, prev_id);
                    }
                }
            }
            false
        }
        Combinator::SubsequentSibling => {
            // Any preceding sibling must match
            let mut current = tree.prev_sibling(node_id);
            while let Some(sibling_id) = current {
                if let Some(node) = tree.get(sibling_id) {
                    if let Some(elem) = node.as_element() {
                        if match_selector(next_selector, elem, tree, sibling_id) {
                            return true;
                        }
                    }
                }
                current = tree.prev_sibling(sibling_id);
            }
            false
        }
    }
}

/// Context for pseudo-class matching.
#[derive(Clone, Debug, Default)]
pub struct MatchContext {
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
    pub visited: bool,
    pub target: bool,
    pub nth_child: usize,
    pub nth_last_child: usize,
    pub nth_of_type: usize,
    pub nth_last_of_type: usize,
    pub is_first_child: bool,
    pub is_last_child: bool,
    pub is_only_child: bool,
    pub is_first_of_type: bool,
    pub is_last_of_type: bool,
    pub is_only_of_type: bool,
    pub is_empty: bool,
    pub is_root: bool,
}

impl MatchContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_element(tree: &DomTree, node_id: NodeId) -> Self {
        let mut ctx = Self::new();

        if let Some(node) = tree.get(node_id) {
            // Check root
            ctx.is_root = tree.parent(node_id).is_none();

            // Check empty
            ctx.is_empty = node.children.is_empty();
        }

        // Get sibling info
        if let Some(parent_id) = tree.parent(node_id) {
            if let Some(parent) = tree.get(parent_id) {
                let siblings: Vec<_> = parent.children.iter().copied().collect();
                let element_siblings: Vec<_> = siblings
                    .iter()
                    .filter(|&&id| tree.get(id).map(|n| n.is_element()).unwrap_or(false))
                    .copied()
                    .collect();

                if let Some(pos) = element_siblings.iter().position(|&id| id == node_id) {
                    ctx.nth_child = pos + 1;
                    ctx.nth_last_child = element_siblings.len() - pos;
                    ctx.is_first_child = pos == 0;
                    ctx.is_last_child = pos == element_siblings.len() - 1;
                    ctx.is_only_child = element_siblings.len() == 1;
                }

                // Get nth-of-type info
                if let Some(node) = tree.get(node_id) {
                    if let Some(elem) = node.as_element() {
                        let same_type: Vec<_> = element_siblings
                            .iter()
                            .filter(|&&id| {
                                tree.get(id)
                                    .and_then(|n| n.as_element())
                                    .map(|e| e.tag_name == elem.tag_name)
                                    .unwrap_or(false)
                            })
                            .copied()
                            .collect();

                        if let Some(pos) = same_type.iter().position(|&id| id == node_id) {
                            ctx.nth_of_type = pos + 1;
                            ctx.nth_last_of_type = same_type.len() - pos;
                            ctx.is_first_of_type = pos == 0;
                            ctx.is_last_of_type = pos == same_type.len() - 1;
                            ctx.is_only_of_type = same_type.len() == 1;
                        }
                    }
                }
            }
        }

        ctx
    }
}

/// Match pseudo-class.
pub fn match_pseudo_class(
    name: &str,
    args: Option<&str>,
    context: &MatchContext,
) -> bool {
    match name {
        "hover" => context.hovered,
        "active" => context.active,
        "focus" => context.focused,
        "visited" => context.visited,
        "link" => !context.visited,
        "target" => context.target,
        "first-child" => context.is_first_child,
        "last-child" => context.is_last_child,
        "only-child" => context.is_only_child,
        "first-of-type" => context.is_first_of_type,
        "last-of-type" => context.is_last_of_type,
        "only-of-type" => context.is_only_of_type,
        "empty" => context.is_empty,
        "root" => context.is_root,
        "nth-child" => match_nth(args, context.nth_child),
        "nth-last-child" => match_nth(args, context.nth_last_child),
        "nth-of-type" => match_nth(args, context.nth_of_type),
        "nth-last-of-type" => match_nth(args, context.nth_last_of_type),
        "enabled" => true, // Would need element state
        "disabled" => false,
        "checked" => false,
        "required" => false,
        "optional" => true,
        _ => true, // Unknown pseudo-classes match by default
    }
}

/// Match nth-* expression.
fn match_nth(args: Option<&str>, n: usize) -> bool {
    let args = match args {
        Some(a) => a.trim(),
        None => return false,
    };

    // Parse An+B syntax
    if args == "odd" {
        return n % 2 == 1;
    }
    if args == "even" {
        return n % 2 == 0;
    }

    // Simple number
    if let Ok(num) = args.parse::<usize>() {
        return n == num;
    }

    // An+B form
    // Simplified parsing - real implementation would be more robust
    if args.contains('n') {
        let parts: Vec<&str> = args.split('n').collect();
        let a: i32 = match parts[0].trim() {
            "" | "+" => 1,
            "-" => -1,
            s => s.parse().unwrap_or(0),
        };
        let b: i32 = parts
            .get(1)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        if a == 0 {
            return n as i32 == b;
        }

        let diff = n as i32 - b;
        if a > 0 {
            diff >= 0 && diff % a == 0
        } else {
            diff <= 0 && diff % a == 0
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::element::TagName;

    #[test]
    fn test_match_tag() {
        let selector = Selector::tag("div");
        let elem = ElementData::new(TagName::div());
        let tree = DomTree::new();

        assert!(match_simple_selector(&selector, &elem));
    }

    #[test]
    fn test_match_class() {
        let selector = Selector::class("container");
        let mut elem = ElementData::new(TagName::div());
        elem.set_attribute("class", "container main");

        assert!(match_simple_selector(&selector, &elem));
    }

    #[test]
    fn test_match_id() {
        let selector = Selector::id("main");
        let mut elem = ElementData::new(TagName::div());
        elem.set_attribute("id", "main");

        assert!(match_simple_selector(&selector, &elem));
    }

    #[test]
    fn test_nth_parsing() {
        assert!(match_nth(Some("odd"), 1));
        assert!(match_nth(Some("odd"), 3));
        assert!(!match_nth(Some("odd"), 2));
        assert!(match_nth(Some("even"), 2));
        assert!(match_nth(Some("3"), 3));
        assert!(match_nth(Some("2n"), 4));
        assert!(match_nth(Some("2n+1"), 3));
    }
}
