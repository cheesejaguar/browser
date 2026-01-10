//! Subresource Integrity (SRI) implementation.

use sha2::{Digest, Sha256, Sha384, Sha512};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Subresource Integrity checker.
#[derive(Clone, Debug, Default)]
pub struct SubresourceIntegrity {
    /// Integrity metadata.
    metadata: Vec<IntegrityMetadata>,
}

impl SubresourceIntegrity {
    /// Create a new SRI checker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an integrity attribute.
    pub fn parse(integrity: &str) -> Self {
        let mut sri = Self::new();

        for token in integrity.split_whitespace() {
            if let Some(metadata) = IntegrityMetadata::parse(token) {
                sri.metadata.push(metadata);
            }
        }

        sri
    }

    /// Check if the content matches the integrity.
    pub fn verify(&self, content: &[u8]) -> bool {
        if self.metadata.is_empty() {
            return true; // No integrity requirement
        }

        // Group by algorithm and find the strongest
        let strongest = self.strongest_algorithm();

        // Content must match at least one hash with the strongest algorithm
        for metadata in &self.metadata {
            if metadata.algorithm == strongest && metadata.matches(content) {
                return true;
            }
        }

        false
    }

    /// Get the strongest algorithm used.
    fn strongest_algorithm(&self) -> HashAlgorithm {
        self.metadata
            .iter()
            .map(|m| m.algorithm)
            .max_by_key(|a| a.strength())
            .unwrap_or(HashAlgorithm::Sha256)
    }

    /// Check if integrity is specified.
    pub fn has_integrity(&self) -> bool {
        !self.metadata.is_empty()
    }

    /// Generate integrity for content.
    pub fn generate(content: &[u8], algorithm: HashAlgorithm) -> String {
        let hash = algorithm.hash(content);
        let encoded = BASE64.encode(&hash);
        format!("{}-{}", algorithm.name(), encoded)
    }
}

/// Integrity metadata.
#[derive(Clone, Debug)]
pub struct IntegrityMetadata {
    /// Hash algorithm.
    pub algorithm: HashAlgorithm,
    /// Base64-encoded hash.
    pub hash: String,
    /// Optional options.
    pub options: Vec<String>,
}

impl IntegrityMetadata {
    /// Parse a single integrity token.
    pub fn parse(token: &str) -> Option<Self> {
        let mut parts = token.splitn(2, '-');
        let algorithm_str = parts.next()?;
        let rest = parts.next()?;

        let algorithm = HashAlgorithm::from_name(algorithm_str)?;

        // Handle options (separated by ?)
        let mut hash_and_options = rest.splitn(2, '?');
        let hash = hash_and_options.next()?.to_string();
        let options: Vec<String> = hash_and_options
            .next()
            .map(|o| o.split('?').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        Some(Self {
            algorithm,
            hash,
            options,
        })
    }

    /// Check if content matches this metadata.
    pub fn matches(&self, content: &[u8]) -> bool {
        let computed_hash = self.algorithm.hash(content);
        let computed_encoded = BASE64.encode(&computed_hash);
        self.hash == computed_encoded
    }
}

/// Hash algorithm for SRI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256.
    Sha256,
    /// SHA-384.
    Sha384,
    /// SHA-512.
    Sha512,
}

impl HashAlgorithm {
    /// Get algorithm name.
    pub fn name(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha256 => "sha256",
            HashAlgorithm::Sha384 => "sha384",
            HashAlgorithm::Sha512 => "sha512",
        }
    }

    /// Parse algorithm from name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "sha256" => Some(HashAlgorithm::Sha256),
            "sha384" => Some(HashAlgorithm::Sha384),
            "sha512" => Some(HashAlgorithm::Sha512),
            _ => None,
        }
    }

    /// Get algorithm strength (higher is stronger).
    pub fn strength(&self) -> u8 {
        match self {
            HashAlgorithm::Sha256 => 1,
            HashAlgorithm::Sha384 => 2,
            HashAlgorithm::Sha512 => 3,
        }
    }

    /// Hash content using this algorithm.
    pub fn hash(&self, content: &[u8]) -> Vec<u8> {
        match self {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(content);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(content);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(content);
                hasher.finalize().to_vec()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sri_parse() {
        let sri = SubresourceIntegrity::parse(
            "sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC",
        );
        assert!(sri.has_integrity());
        assert_eq!(sri.metadata.len(), 1);
        assert_eq!(sri.metadata[0].algorithm, HashAlgorithm::Sha384);
    }

    #[test]
    fn test_sri_multiple_hashes() {
        let sri = SubresourceIntegrity::parse(
            "sha256-abc123 sha384-def456 sha512-ghi789",
        );
        assert_eq!(sri.metadata.len(), 3);
    }

    #[test]
    fn test_sri_verify() {
        let content = b"hello world";
        let hash = SubresourceIntegrity::generate(content, HashAlgorithm::Sha256);

        let sri = SubresourceIntegrity::parse(&hash);
        assert!(sri.verify(content));
        assert!(!sri.verify(b"different content"));
    }

    #[test]
    fn test_sri_generate() {
        let content = b"test content";
        let integrity = SubresourceIntegrity::generate(content, HashAlgorithm::Sha256);
        assert!(integrity.starts_with("sha256-"));
    }

    #[test]
    fn test_hash_algorithm_strength() {
        assert!(HashAlgorithm::Sha512.strength() > HashAlgorithm::Sha384.strength());
        assert!(HashAlgorithm::Sha384.strength() > HashAlgorithm::Sha256.strength());
    }
}
