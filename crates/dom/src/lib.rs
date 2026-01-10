//! DOM (Document Object Model) implementation.
//!
//! This crate provides the core DOM tree structure used by the browser engine.

pub mod node;
pub mod document;
pub mod element;
pub mod text;
pub mod comment;
pub mod tree;
pub mod events;
pub mod attributes;
pub mod window;

pub use node::{Node, NodeId, NodeType, NodeData};
pub use document::Document;
pub use element::{Element, ElementData, TagName};
pub use text::Text;
pub use comment::Comment;
pub use tree::DomTree;
pub use events::{Event, EventType, EventTarget, EventPhase};
pub use attributes::{Attribute, AttributeMap};
pub use window::Window;
