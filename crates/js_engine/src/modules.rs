//! JavaScript module system (ES Modules).

use boa_engine::{Context, JsResult, JsValue, Module, Source, js_string};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

/// Module loader for ES modules.
pub struct ModuleLoader {
    /// Loaded modules by URL.
    modules: HashMap<String, LoadedModule>,
    /// Module graph.
    graph: ModuleGraph,
    /// Base URL for relative imports.
    base_url: Option<String>,
}

impl ModuleLoader {
    /// Create a new module loader.
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            graph: ModuleGraph::new(),
            base_url: None,
        }
    }

    /// Set the base URL for relative imports.
    pub fn set_base_url(&mut self, url: impl Into<String>) {
        self.base_url = Some(url.into());
    }

    /// Load a module from source.
    pub fn load_module(
        &mut self,
        url: &str,
        source: &str,
        context: &mut Context,
    ) -> Result<ModuleId, ModuleError> {
        // Check if already loaded
        if let Some(module) = self.modules.get(url) {
            return Ok(module.id);
        }

        // Parse the module
        let source = Source::from_bytes(source.as_bytes());

        let module = Module::parse(source, None, context)
            .map_err(|e| ModuleError::Parse(e.to_string()))?;

        // Create module record
        let id = ModuleId(self.modules.len() as u64);
        let loaded = LoadedModule {
            id,
            url: url.to_string(),
            module,
            state: ModuleState::Parsed,
            dependencies: Vec::new(),
        };

        self.modules.insert(url.to_string(), loaded);
        self.graph.add_node(id, url.to_string());

        Ok(id)
    }

    /// Link a module (resolve dependencies).
    pub fn link_module(
        &mut self,
        id: ModuleId,
        context: &mut Context,
    ) -> Result<(), ModuleError> {
        let module = self
            .modules
            .get_mut(&self.get_url(id).unwrap_or_default())
            .ok_or(ModuleError::NotFound)?;

        if module.state != ModuleState::Parsed {
            return Ok(()); // Already linked or evaluated
        }

        // Get module requests (imports)
        // In a real implementation, we'd parse the AST to find imports
        // For now, mark as linked
        module.state = ModuleState::Linked;

        Ok(())
    }

    /// Evaluate a module.
    pub fn evaluate_module(
        &mut self,
        id: ModuleId,
        context: &mut Context,
    ) -> Result<JsValue, ModuleError> {
        let url = self.get_url(id).ok_or(ModuleError::NotFound)?;
        let module = self
            .modules
            .get_mut(&url)
            .ok_or(ModuleError::NotFound)?;

        if module.state == ModuleState::Evaluated {
            // Return cached namespace
            return Ok(JsValue::undefined()); // Would return namespace object
        }

        // Ensure linked
        if module.state == ModuleState::Parsed {
            self.link_module(id, context)?;
        }

        // Evaluate the module
        let module_obj = module.module.clone();
        module.state = ModuleState::Evaluating;

        // Load and link the module in Boa
        module_obj.load_link_evaluate(context);

        // Run jobs to complete evaluation
        context.run_jobs();

        let url = self.get_url(id).ok_or(ModuleError::NotFound)?;
        if let Some(module) = self.modules.get_mut(&url) {
            module.state = ModuleState::Evaluated;
        }

        Ok(JsValue::undefined())
    }

    /// Get a module by ID.
    pub fn get_module(&self, id: ModuleId) -> Option<&LoadedModule> {
        self.modules.values().find(|m| m.id == id)
    }

    /// Get module URL by ID.
    fn get_url(&self, id: ModuleId) -> Option<String> {
        self.modules
            .values()
            .find(|m| m.id == id)
            .map(|m| m.url.clone())
    }

    /// Resolve a module specifier to a URL.
    pub fn resolve(&self, specifier: &str, referrer: Option<&str>) -> Result<String, ModuleError> {
        // Handle bare specifiers (npm packages, etc.)
        if !specifier.starts_with('.') && !specifier.starts_with('/') && !specifier.contains(':') {
            // Bare specifier - would need import map or node_modules resolution
            return Err(ModuleError::BareSpecifier(specifier.to_string()));
        }

        // Handle relative specifiers
        if specifier.starts_with('.') {
            let base = referrer.or(self.base_url.as_deref()).ok_or_else(|| {
                ModuleError::Resolution(format!(
                    "Cannot resolve relative specifier '{}' without a base URL",
                    specifier
                ))
            })?;

            return resolve_relative(specifier, base);
        }

        // Absolute URL
        Ok(specifier.to_string())
    }

    /// Clear all loaded modules.
    pub fn clear(&mut self) {
        self.modules.clear();
        self.graph = ModuleGraph::new();
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve a relative module specifier.
fn resolve_relative(specifier: &str, base: &str) -> Result<String, ModuleError> {
    // Simple URL resolution
    if let Ok(base_url) = url::Url::parse(base) {
        base_url
            .join(specifier)
            .map(|u| u.to_string())
            .map_err(|e| ModuleError::Resolution(e.to_string()))
    } else {
        // File path resolution
        let base_path = PathBuf::from(base);
        let parent = base_path.parent().unwrap_or(&base_path);
        let resolved = parent.join(specifier);

        Ok(resolved.to_string_lossy().to_string())
    }
}

/// Module identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u64);

/// A loaded module.
pub struct LoadedModule {
    /// Module ID.
    pub id: ModuleId,
    /// Module URL.
    pub url: String,
    /// Boa module.
    pub module: Module,
    /// Module state.
    pub state: ModuleState,
    /// Dependencies (imports).
    pub dependencies: Vec<ModuleId>,
}

/// Module state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleState {
    /// Module has been parsed.
    Parsed,
    /// Module is being linked.
    Linking,
    /// Module has been linked.
    Linked,
    /// Module is being evaluated.
    Evaluating,
    /// Module has been evaluated.
    Evaluated,
    /// Module evaluation failed.
    Error,
}

/// Module graph for dependency tracking.
pub struct ModuleGraph {
    /// Nodes (module ID -> URL).
    nodes: HashMap<ModuleId, String>,
    /// Edges (module ID -> dependencies).
    edges: HashMap<ModuleId, Vec<ModuleId>>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, id: ModuleId, url: String) {
        self.nodes.insert(id, url);
        self.edges.entry(id).or_insert_with(Vec::new);
    }

    /// Add an edge (dependency).
    pub fn add_edge(&mut self, from: ModuleId, to: ModuleId) {
        self.edges.entry(from).or_insert_with(Vec::new).push(to);
    }

    /// Get dependencies of a module.
    pub fn dependencies(&self, id: ModuleId) -> &[ModuleId] {
        self.edges.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check for cycles.
    pub fn has_cycle(&self) -> bool {
        // Simple DFS cycle detection
        let mut visited = std::collections::HashSet::new();
        let mut stack = std::collections::HashSet::new();

        for &id in self.nodes.keys() {
            if self.has_cycle_from(id, &mut visited, &mut stack) {
                return true;
            }
        }

        false
    }

    fn has_cycle_from(
        &self,
        id: ModuleId,
        visited: &mut std::collections::HashSet<ModuleId>,
        stack: &mut std::collections::HashSet<ModuleId>,
    ) -> bool {
        if stack.contains(&id) {
            return true;
        }
        if visited.contains(&id) {
            return false;
        }

        visited.insert(id);
        stack.insert(id);

        for &dep in self.dependencies(id) {
            if self.has_cycle_from(dep, visited, stack) {
                return true;
            }
        }

        stack.remove(&id);
        false
    }

    /// Get topological order for evaluation.
    pub fn topological_order(&self) -> Result<Vec<ModuleId>, ModuleError> {
        if self.has_cycle() {
            return Err(ModuleError::CyclicDependency);
        }

        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for &id in self.nodes.keys() {
            self.visit(id, &mut visited, &mut result);
        }

        result.reverse();
        Ok(result)
    }

    fn visit(
        &self,
        id: ModuleId,
        visited: &mut std::collections::HashSet<ModuleId>,
        result: &mut Vec<ModuleId>,
    ) {
        if visited.contains(&id) {
            return;
        }

        visited.insert(id);

        for &dep in self.dependencies(id) {
            self.visit(dep, visited, result);
        }

        result.push(id);
    }
}

impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Module error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ModuleError {
    #[error("Module not found")]
    NotFound,
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Link error: {0}")]
    Link(String),
    #[error("Evaluation error: {0}")]
    Evaluation(String),
    #[error("Resolution error: {0}")]
    Resolution(String),
    #[error("Bare specifier not supported: {0}")]
    BareSpecifier(String),
    #[error("Cyclic dependency detected")]
    CyclicDependency,
}

/// Import map for bare specifier resolution.
#[derive(Clone, Debug, Default)]
pub struct ImportMap {
    /// Import mappings.
    imports: HashMap<String, String>,
    /// Scoped mappings.
    scopes: HashMap<String, HashMap<String, String>>,
}

impl ImportMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an import mapping.
    pub fn add_import(&mut self, specifier: impl Into<String>, url: impl Into<String>) {
        self.imports.insert(specifier.into(), url.into());
    }

    /// Add a scoped mapping.
    pub fn add_scoped(
        &mut self,
        scope: impl Into<String>,
        specifier: impl Into<String>,
        url: impl Into<String>,
    ) {
        self.scopes
            .entry(scope.into())
            .or_insert_with(HashMap::new)
            .insert(specifier.into(), url.into());
    }

    /// Resolve a specifier using the import map.
    pub fn resolve(&self, specifier: &str, referrer: Option<&str>) -> Option<String> {
        // Check scoped mappings first
        if let Some(referrer) = referrer {
            for (scope, mappings) in &self.scopes {
                if referrer.starts_with(scope) {
                    if let Some(url) = mappings.get(specifier) {
                        return Some(url.clone());
                    }
                }
            }
        }

        // Check global mappings
        self.imports.get(specifier).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert!(loader.modules.is_empty());
    }

    #[test]
    fn test_resolve_relative() {
        let result = resolve_relative("./module.js", "https://example.com/app/main.js").unwrap();
        assert_eq!(result, "https://example.com/app/module.js");
    }

    #[test]
    fn test_module_graph_cycle_detection() {
        let mut graph = ModuleGraph::new();
        graph.add_node(ModuleId(1), "a.js".to_string());
        graph.add_node(ModuleId(2), "b.js".to_string());
        graph.add_node(ModuleId(3), "c.js".to_string());

        graph.add_edge(ModuleId(1), ModuleId(2));
        graph.add_edge(ModuleId(2), ModuleId(3));
        assert!(!graph.has_cycle());

        graph.add_edge(ModuleId(3), ModuleId(1));
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_import_map() {
        let mut import_map = ImportMap::new();
        import_map.add_import("lodash", "https://cdn.example.com/lodash.js");

        let resolved = import_map.resolve("lodash", None);
        assert_eq!(resolved, Some("https://cdn.example.com/lodash.js".to_string()));
    }
}
