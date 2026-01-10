//! Iframe sandboxing implementation.

use std::collections::HashSet;

/// Sandbox for iframe and document restrictions.
#[derive(Clone, Debug)]
pub struct Sandbox {
    /// Active sandbox flags.
    flags: SandboxFlags,
    /// Custom allowed features.
    allowed_features: HashSet<String>,
}

impl Sandbox {
    /// Create a new sandbox with all restrictions enabled.
    pub fn new() -> Self {
        Self {
            flags: SandboxFlags::all(),
            allowed_features: HashSet::new(),
        }
    }

    /// Create a sandbox with no restrictions (not sandboxed).
    pub fn none() -> Self {
        Self {
            flags: SandboxFlags::empty(),
            allowed_features: HashSet::new(),
        }
    }

    /// Parse sandbox attribute value.
    pub fn parse(attribute: &str) -> Self {
        let mut sandbox = Self::new();

        for token in attribute.split_whitespace() {
            match token {
                "allow-forms" => sandbox.flags.remove(SandboxFlags::FORMS),
                "allow-modals" => sandbox.flags.remove(SandboxFlags::MODALS),
                "allow-orientation-lock" => sandbox.flags.remove(SandboxFlags::ORIENTATION_LOCK),
                "allow-pointer-lock" => sandbox.flags.remove(SandboxFlags::POINTER_LOCK),
                "allow-popups" => sandbox.flags.remove(SandboxFlags::POPUPS),
                "allow-popups-to-escape-sandbox" => {
                    sandbox.flags.remove(SandboxFlags::POPUPS_TO_ESCAPE_SANDBOX)
                }
                "allow-presentation" => sandbox.flags.remove(SandboxFlags::PRESENTATION),
                "allow-same-origin" => sandbox.flags.remove(SandboxFlags::SAME_ORIGIN),
                "allow-scripts" => sandbox.flags.remove(SandboxFlags::SCRIPTS),
                "allow-top-navigation" => sandbox.flags.remove(SandboxFlags::TOP_NAVIGATION),
                "allow-top-navigation-by-user-activation" => {
                    sandbox.flags.remove(SandboxFlags::TOP_NAVIGATION_BY_USER_ACTIVATION)
                }
                "allow-downloads" => sandbox.flags.remove(SandboxFlags::DOWNLOADS),
                _ => {
                    // Unknown tokens are ignored per spec
                }
            }
        }

        sandbox
    }

    /// Check if forms are allowed.
    pub fn allows_forms(&self) -> bool {
        !self.flags.contains(SandboxFlags::FORMS)
    }

    /// Check if modals (alert, confirm, prompt) are allowed.
    pub fn allows_modals(&self) -> bool {
        !self.flags.contains(SandboxFlags::MODALS)
    }

    /// Check if orientation lock is allowed.
    pub fn allows_orientation_lock(&self) -> bool {
        !self.flags.contains(SandboxFlags::ORIENTATION_LOCK)
    }

    /// Check if pointer lock is allowed.
    pub fn allows_pointer_lock(&self) -> bool {
        !self.flags.contains(SandboxFlags::POINTER_LOCK)
    }

    /// Check if popups are allowed.
    pub fn allows_popups(&self) -> bool {
        !self.flags.contains(SandboxFlags::POPUPS)
    }

    /// Check if popups should escape sandbox.
    pub fn popups_escape_sandbox(&self) -> bool {
        !self.flags.contains(SandboxFlags::POPUPS_TO_ESCAPE_SANDBOX)
    }

    /// Check if presentation is allowed.
    pub fn allows_presentation(&self) -> bool {
        !self.flags.contains(SandboxFlags::PRESENTATION)
    }

    /// Check if same-origin is preserved.
    pub fn allows_same_origin(&self) -> bool {
        !self.flags.contains(SandboxFlags::SAME_ORIGIN)
    }

    /// Check if scripts are allowed.
    pub fn allows_scripts(&self) -> bool {
        !self.flags.contains(SandboxFlags::SCRIPTS)
    }

    /// Check if top-level navigation is allowed.
    pub fn allows_top_navigation(&self) -> bool {
        !self.flags.contains(SandboxFlags::TOP_NAVIGATION)
    }

    /// Check if top-level navigation by user activation is allowed.
    pub fn allows_top_navigation_by_user_activation(&self) -> bool {
        !self.flags.contains(SandboxFlags::TOP_NAVIGATION_BY_USER_ACTIVATION)
    }

    /// Check if downloads are allowed.
    pub fn allows_downloads(&self) -> bool {
        !self.flags.contains(SandboxFlags::DOWNLOADS)
    }

    /// Check if a specific feature is allowed.
    pub fn allows_feature(&self, feature: &str) -> bool {
        self.allowed_features.contains(feature)
    }

    /// Get the sandbox flags.
    pub fn flags(&self) -> SandboxFlags {
        self.flags
    }

    /// Check if sandboxed.
    pub fn is_sandboxed(&self) -> bool {
        !self.flags.is_empty()
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::none()
    }
}

bitflags::bitflags! {
    /// Sandbox flags.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SandboxFlags: u32 {
        /// Block form submissions.
        const FORMS = 1 << 0;
        /// Block modals (alert, confirm, prompt).
        const MODALS = 1 << 1;
        /// Block orientation lock.
        const ORIENTATION_LOCK = 1 << 2;
        /// Block pointer lock.
        const POINTER_LOCK = 1 << 3;
        /// Block popups.
        const POPUPS = 1 << 4;
        /// Popups inherit sandbox.
        const POPUPS_TO_ESCAPE_SANDBOX = 1 << 5;
        /// Block presentation.
        const PRESENTATION = 1 << 6;
        /// Treat as unique origin.
        const SAME_ORIGIN = 1 << 7;
        /// Block scripts.
        const SCRIPTS = 1 << 8;
        /// Block top-level navigation.
        const TOP_NAVIGATION = 1 << 9;
        /// Block top-level navigation without user activation.
        const TOP_NAVIGATION_BY_USER_ACTIVATION = 1 << 10;
        /// Block downloads.
        const DOWNLOADS = 1 << 11;
    }
}

/// Document sandbox state.
#[derive(Clone, Debug)]
pub struct DocumentSandbox {
    /// The sandbox configuration.
    sandbox: Sandbox,
    /// Whether this is an iframe.
    is_iframe: bool,
    /// Parent sandbox (for nested iframes).
    parent_sandbox: Option<Box<DocumentSandbox>>,
}

impl DocumentSandbox {
    /// Create a new document sandbox.
    pub fn new(sandbox: Sandbox, is_iframe: bool) -> Self {
        Self {
            sandbox,
            is_iframe,
            parent_sandbox: None,
        }
    }

    /// Set the parent sandbox.
    pub fn set_parent(&mut self, parent: DocumentSandbox) {
        self.parent_sandbox = Some(Box::new(parent));
    }

    /// Check if an action is allowed, considering parent sandboxes.
    pub fn allows_action(&self, check: impl Fn(&Sandbox) -> bool) -> bool {
        // Check this sandbox
        if !check(&self.sandbox) {
            return false;
        }

        // Check parent sandboxes
        if let Some(ref parent) = self.parent_sandbox {
            return parent.allows_action(check);
        }

        true
    }

    /// Get effective sandbox flags.
    pub fn effective_flags(&self) -> SandboxFlags {
        let mut flags = self.sandbox.flags();

        if let Some(ref parent) = self.parent_sandbox {
            flags |= parent.effective_flags();
        }

        flags
    }
}

/// Process sandbox for security isolation.
#[derive(Clone, Debug)]
pub struct ProcessSandbox {
    /// Whether sandboxing is enabled.
    enabled: bool,
    /// Process restrictions.
    restrictions: ProcessRestrictions,
}

impl ProcessSandbox {
    /// Create a new process sandbox.
    pub fn new() -> Self {
        Self {
            enabled: true,
            restrictions: ProcessRestrictions::default(),
        }
    }

    /// Disable sandboxing (for debugging).
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if sandboxing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the restrictions.
    pub fn restrictions(&self) -> &ProcessRestrictions {
        &self.restrictions
    }
}

impl Default for ProcessSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Process-level restrictions.
#[derive(Clone, Debug)]
pub struct ProcessRestrictions {
    /// Allow network access.
    pub allow_network: bool,
    /// Allow filesystem access.
    pub allow_filesystem: bool,
    /// Allow GPU access.
    pub allow_gpu: bool,
    /// Allow audio access.
    pub allow_audio: bool,
    /// Allow clipboard access.
    pub allow_clipboard: bool,
    /// Allowed paths for filesystem access.
    pub allowed_paths: Vec<String>,
}

impl Default for ProcessRestrictions {
    fn default() -> Self {
        Self {
            allow_network: true,
            allow_filesystem: false,
            allow_gpu: true,
            allow_audio: true,
            allow_clipboard: false,
            allowed_paths: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_parse_empty() {
        let sandbox = Sandbox::parse("");
        assert!(sandbox.is_sandboxed());
        assert!(!sandbox.allows_scripts());
        assert!(!sandbox.allows_same_origin());
        assert!(!sandbox.allows_forms());
    }

    #[test]
    fn test_sandbox_parse_allow_scripts() {
        let sandbox = Sandbox::parse("allow-scripts");
        assert!(sandbox.is_sandboxed());
        assert!(sandbox.allows_scripts());
        assert!(!sandbox.allows_same_origin());
    }

    #[test]
    fn test_sandbox_parse_multiple() {
        let sandbox = Sandbox::parse("allow-scripts allow-same-origin allow-forms");
        assert!(sandbox.allows_scripts());
        assert!(sandbox.allows_same_origin());
        assert!(sandbox.allows_forms());
        assert!(!sandbox.allows_popups());
    }

    #[test]
    fn test_document_sandbox_inheritance() {
        let parent = DocumentSandbox::new(Sandbox::parse("allow-scripts"), true);
        let mut child = DocumentSandbox::new(Sandbox::parse("allow-scripts allow-forms"), true);
        child.set_parent(parent);

        // Child allows forms, but check inheritance
        assert!(child.allows_action(|s| s.allows_scripts()));
        assert!(child.allows_action(|s| s.allows_forms()));
    }
}
