//! Engine version identity for save-file and checkpoint compatibility.
//!
//! An [`EngineVersion`] bundles three pieces of information:
//!
//! - **Semantic version** — the workspace version from `Cargo.toml`, bumped
//!   deliberately at release time.
//! - **Git commit** — the full commit hash at build time, for traceability.
//! - **Content hash** — a SHA-256 digest of all gameplay-relevant definitions
//!   (cards, enemies, subclasses, etc.), computed at runtime. This catches
//!   content changes that haven't been formally versioned yet.
//!
//! Two engine versions are **compatible** if and only if both `semver` and
//! `content_hash` match. The git commit is informational only.

use serde::{Deserialize, Serialize};

pub const SEMVER: &str = env!("CARGO_PKG_VERSION");
pub const GIT_COMMIT: &str = env!("DECKER_GIT_COMMIT");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineVersion {
    pub semver: String,
    pub git_commit: String,
    pub content_hash: String,
}

impl EngineVersion {
    /// Build a version with the given content hash.
    ///
    /// The semver and git commit are baked in at compile time; the content
    /// hash is supplied by the caller (typically computed by `decker-content`).
    pub fn new(content_hash: String) -> Self {
        Self {
            semver: SEMVER.to_string(),
            git_commit: GIT_COMMIT.to_string(),
            content_hash,
        }
    }

    /// Check whether two versions are compatible (same semver + content hash).
    pub fn is_compatible_with(&self, other: &EngineVersion) -> bool {
        self.semver == other.semver && self.content_hash == other.content_hash
    }
}

impl std::fmt::Display for EngineVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "v{} (commit: {}, content: {})",
            self.semver,
            &self.git_commit[..self.git_commit.len().min(8)],
            &self.content_hash[..self.content_hash.len().min(12)],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_matches_cargo_toml() {
        assert!(!SEMVER.is_empty());
        assert!(SEMVER.contains('.'), "semver should be dotted: {SEMVER}");
    }

    #[test]
    fn compatibility_check() {
        let a = EngineVersion::new("abc123".into());
        let b = EngineVersion::new("abc123".into());
        let c = EngineVersion::new("def456".into());

        assert!(a.is_compatible_with(&b));
        assert!(!a.is_compatible_with(&c));
    }

    #[test]
    fn display_format() {
        let v = EngineVersion::new("abcdef1234567890".into());
        let s = format!("{v}");
        assert!(s.contains(SEMVER));
        assert!(s.contains("abcdef123456"));
    }
}
