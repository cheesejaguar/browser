//! Browser security features.
//!
//! This crate implements core browser security mechanisms:
//! - Same-Origin Policy (SOP)
//! - Content Security Policy (CSP)
//! - Cross-Origin Resource Sharing (CORS)
//! - Sandboxing
//! - Secure contexts
//! - Mixed content blocking

pub mod origin;
pub mod csp;
pub mod cors;
pub mod sandbox;
pub mod mixed_content;
pub mod secure_context;
pub mod permissions;
pub mod sri;

pub use origin::{Origin, OriginPolicy};
pub use csp::{ContentSecurityPolicy, CspDirective, CspViolation};
pub use cors::{CorsConfig, CorsRequest, CorsResponse};
pub use sandbox::{Sandbox, SandboxFlags};
pub use mixed_content::{MixedContentBlocker, MixedContentType};
pub use secure_context::SecureContext;
pub use permissions::{Permission, PermissionState, PermissionsPolicy};
pub use sri::{SubresourceIntegrity, IntegrityMetadata};
