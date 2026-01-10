//! Mutation Observer API implementation.

use std::collections::VecDeque;

/// Mutation Observer.
pub struct MutationObserver {
    /// Observer ID.
    id: u64,
    /// Callback function reference.
    callback: u64,
    /// Observed targets and their options.
    targets: Vec<(u64, MutationObserverInit)>,
    /// Pending mutation records.
    pending_records: VecDeque<MutationRecord>,
    /// Whether the observer is connected.
    connected: bool,
}

impl MutationObserver {
    /// Create a new Mutation Observer.
    pub fn new(callback: u64) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };

        Self {
            id,
            callback,
            targets: Vec::new(),
            pending_records: VecDeque::new(),
            connected: false,
        }
    }

    /// Observe a target node.
    pub fn observe(&mut self, target: u64, options: MutationObserverInit) -> Result<(), String> {
        // Validate options
        if !options.child_list && !options.attributes && !options.character_data {
            return Err("At least one of childList, attributes, or characterData must be true".into());
        }

        if options.attribute_old_value && !options.attributes {
            return Err("attributeOldValue requires attributes to be true".into());
        }

        if options.character_data_old_value && !options.character_data {
            return Err("characterDataOldValue requires characterData to be true".into());
        }

        // Remove existing observation of this target
        self.targets.retain(|(t, _)| *t != target);

        // Add new observation
        self.targets.push((target, options));
        self.connected = true;

        Ok(())
    }

    /// Stop observing all targets.
    pub fn disconnect(&mut self) {
        self.targets.clear();
        self.connected = false;
    }

    /// Take pending records.
    pub fn take_records(&mut self) -> Vec<MutationRecord> {
        self.pending_records.drain(..).collect()
    }

    /// Queue a mutation record.
    pub fn queue_record(&mut self, record: MutationRecord) {
        self.pending_records.push_back(record);
    }

    /// Check if the observer is observing a target.
    pub fn is_observing(&self, target: u64) -> bool {
        self.targets.iter().any(|(t, _)| *t == target)
    }

    /// Get options for a target.
    pub fn get_options(&self, target: u64) -> Option<&MutationObserverInit> {
        self.targets.iter().find(|(t, _)| *t == target).map(|(_, o)| o)
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the callback.
    pub fn callback(&self) -> u64 {
        self.callback
    }
}

/// Mutation observer initialization options.
#[derive(Clone, Debug, Default)]
pub struct MutationObserverInit {
    /// Observe child list changes.
    pub child_list: bool,
    /// Observe attribute changes.
    pub attributes: bool,
    /// Observe character data changes.
    pub character_data: bool,
    /// Observe entire subtree.
    pub subtree: bool,
    /// Record old attribute values.
    pub attribute_old_value: bool,
    /// Record old character data values.
    pub character_data_old_value: bool,
    /// Filter to specific attributes.
    pub attribute_filter: Option<Vec<String>>,
}

impl MutationObserverInit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child_list(mut self) -> Self {
        self.child_list = true;
        self
    }

    pub fn attributes(mut self) -> Self {
        self.attributes = true;
        self
    }

    pub fn character_data(mut self) -> Self {
        self.character_data = true;
        self
    }

    pub fn subtree(mut self) -> Self {
        self.subtree = true;
        self
    }

    pub fn attribute_old_value(mut self) -> Self {
        self.attribute_old_value = true;
        self.attributes = true;
        self
    }

    pub fn character_data_old_value(mut self) -> Self {
        self.character_data_old_value = true;
        self.character_data = true;
        self
    }

    pub fn attribute_filter(mut self, filter: Vec<String>) -> Self {
        self.attribute_filter = Some(filter);
        self.attributes = true;
        self
    }
}

/// Mutation record.
#[derive(Clone, Debug)]
pub struct MutationRecord {
    /// Type of mutation.
    pub mutation_type: MutationType,
    /// Target node.
    pub target: u64,
    /// Added nodes.
    pub added_nodes: Vec<u64>,
    /// Removed nodes.
    pub removed_nodes: Vec<u64>,
    /// Previous sibling.
    pub previous_sibling: Option<u64>,
    /// Next sibling.
    pub next_sibling: Option<u64>,
    /// Attribute name (for attribute mutations).
    pub attribute_name: Option<String>,
    /// Attribute namespace.
    pub attribute_namespace: Option<String>,
    /// Old value.
    pub old_value: Option<String>,
}

impl MutationRecord {
    /// Create a child list mutation record.
    pub fn child_list(target: u64) -> Self {
        Self {
            mutation_type: MutationType::ChildList,
            target,
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: None,
            attribute_namespace: None,
            old_value: None,
        }
    }

    /// Create an attribute mutation record.
    pub fn attributes(target: u64, attribute_name: &str, old_value: Option<String>) -> Self {
        Self {
            mutation_type: MutationType::Attributes,
            target,
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: Some(attribute_name.to_string()),
            attribute_namespace: None,
            old_value,
        }
    }

    /// Create a character data mutation record.
    pub fn character_data(target: u64, old_value: Option<String>) -> Self {
        Self {
            mutation_type: MutationType::CharacterData,
            target,
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: None,
            attribute_namespace: None,
            old_value,
        }
    }

    /// Add an added node.
    pub fn with_added_node(mut self, node: u64) -> Self {
        self.added_nodes.push(node);
        self
    }

    /// Add a removed node.
    pub fn with_removed_node(mut self, node: u64) -> Self {
        self.removed_nodes.push(node);
        self
    }

    /// Set siblings.
    pub fn with_siblings(mut self, previous: Option<u64>, next: Option<u64>) -> Self {
        self.previous_sibling = previous;
        self.next_sibling = next;
        self
    }
}

/// Mutation type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationType {
    ChildList,
    Attributes,
    CharacterData,
}

impl std::fmt::Display for MutationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationType::ChildList => write!(f, "childList"),
            MutationType::Attributes => write!(f, "attributes"),
            MutationType::CharacterData => write!(f, "characterData"),
        }
    }
}

/// Mutation observer controller.
pub struct MutationObserverController {
    /// Active observers.
    observers: Vec<MutationObserver>,
}

impl MutationObserverController {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    /// Register an observer.
    pub fn register(&mut self, observer: MutationObserver) {
        self.observers.push(observer);
    }

    /// Notify observers of a mutation.
    pub fn notify(&mut self, record: MutationRecord) {
        for observer in &mut self.observers {
            if !observer.is_connected() {
                continue;
            }

            // Check if this observer is interested in this target
            let interested = observer.targets.iter().any(|(target, options)| {
                if *target != record.target {
                    // Check subtree
                    if !options.subtree {
                        return false;
                    }
                    // Would need to check if record.target is descendant of target
                }

                match record.mutation_type {
                    MutationType::ChildList => options.child_list,
                    MutationType::Attributes => {
                        if !options.attributes {
                            return false;
                        }
                        // Check attribute filter
                        if let Some(ref filter) = options.attribute_filter {
                            if let Some(ref attr_name) = record.attribute_name {
                                return filter.contains(attr_name);
                            }
                        }
                        true
                    }
                    MutationType::CharacterData => options.character_data,
                }
            });

            if interested {
                observer.queue_record(record.clone());
            }
        }
    }

    /// Invoke callbacks for pending records.
    pub fn flush(&mut self) {
        for observer in &mut self.observers {
            let records = observer.take_records();
            if !records.is_empty() {
                // Would invoke callback with records
            }
        }
    }
}

impl Default for MutationObserverController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_observer_creation() {
        let observer = MutationObserver::new(1);
        assert!(!observer.is_connected());
    }

    #[test]
    fn test_mutation_observer_observe() {
        let mut observer = MutationObserver::new(1);

        let options = MutationObserverInit::new().child_list().subtree();
        assert!(observer.observe(100, options).is_ok());
        assert!(observer.is_connected());
        assert!(observer.is_observing(100));
    }

    #[test]
    fn test_mutation_observer_invalid_options() {
        let mut observer = MutationObserver::new(1);

        // No observation types
        let options = MutationObserverInit::new();
        assert!(observer.observe(100, options).is_err());

        // attributeOldValue without attributes
        let options = MutationObserverInit {
            attribute_old_value: true,
            ..Default::default()
        };
        assert!(observer.observe(100, options).is_err());
    }

    #[test]
    fn test_mutation_record() {
        let record = MutationRecord::child_list(100)
            .with_added_node(101)
            .with_removed_node(102)
            .with_siblings(Some(99), Some(103));

        assert_eq!(record.mutation_type, MutationType::ChildList);
        assert_eq!(record.target, 100);
        assert_eq!(record.added_nodes, vec![101]);
        assert_eq!(record.removed_nodes, vec![102]);
        assert_eq!(record.previous_sibling, Some(99));
        assert_eq!(record.next_sibling, Some(103));
    }

    #[test]
    fn test_take_records() {
        let mut observer = MutationObserver::new(1);

        observer.queue_record(MutationRecord::child_list(100));
        observer.queue_record(MutationRecord::attributes(100, "class", None));

        let records = observer.take_records();
        assert_eq!(records.len(), 2);

        // Should be empty now
        let records = observer.take_records();
        assert!(records.is_empty());
    }
}
