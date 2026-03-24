//! Recent files/projects list with cap and persistence.
//!
//! Extracted from shruti's `Preferences.recent_sessions` pattern.
//! Maintains a most-recently-used list with deduplication, configurable
//! max size, and serde support for persistence across sessions.

use std::path::{Path, PathBuf};

/// A capped, most-recently-used file list.
///
/// Adding a path moves it to the front (most recent). Duplicates are
/// automatically removed. The list is capped at `max_entries`.
/// Serializable for persistence in preferences files.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecentFiles {
    entries: Vec<PathBuf>,
    max_entries: usize,
}

/// Default maximum recent entries.
const DEFAULT_MAX_RECENT: usize = 10;

impl RecentFiles {
    /// Create an empty recent files list with default max (10).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: DEFAULT_MAX_RECENT,
        }
    }

    /// Create an empty recent files list with a custom max.
    #[must_use]
    pub fn with_max(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Add a path to the front of the list.
    ///
    /// If the path already exists, it is moved to the front.
    /// If the list exceeds max, the oldest entry is removed.
    pub fn add(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.entries.retain(|p| p != &path);
        self.entries.insert(0, path);
        self.entries.truncate(self.max_entries);
        tracing::debug!(count = self.entries.len(), "recent file added");
    }

    /// Remove a specific path from the list.
    pub fn remove(&mut self, path: &Path) {
        let before = self.entries.len();
        self.entries.retain(|p| p != path);
        if self.entries.len() < before {
            tracing::debug!(count = self.entries.len(), "recent file removed");
        }
    }

    /// All entries, most recent first.
    #[must_use]
    #[inline]
    pub fn entries(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Most recent path.
    #[must_use]
    #[inline]
    pub fn most_recent(&self) -> Option<&Path> {
        self.entries.first().map(|p| p.as_path())
    }

    /// Number of entries.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the list is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Maximum number of entries.
    #[must_use]
    #[inline]
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        tracing::debug!("recent files cleared");
    }

    /// Remove entries whose paths no longer exist on disk.
    pub fn prune_missing(&mut self) {
        let before = self.entries.len();
        self.entries.retain(|p| p.exists());
        let removed = before - self.entries.len();
        if removed > 0 {
            tracing::debug!(removed, "pruned missing recent files");
        }
    }
}

impl Default for RecentFiles {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let recent = RecentFiles::new();
        assert!(recent.is_empty());
        assert_eq!(recent.max_entries(), DEFAULT_MAX_RECENT);
        assert!(recent.most_recent().is_none());
    }

    #[test]
    fn add_and_query() {
        let mut recent = RecentFiles::new();
        recent.add("/projects/a.proj");
        recent.add("/projects/b.proj");

        assert_eq!(recent.len(), 2);
        assert_eq!(recent.most_recent(), Some(Path::new("/projects/b.proj")));
        assert_eq!(recent.entries()[1], PathBuf::from("/projects/a.proj"));
    }

    #[test]
    fn add_deduplicates_and_moves_to_front() {
        let mut recent = RecentFiles::new();
        recent.add("/a");
        recent.add("/b");
        recent.add("/c");
        recent.add("/a"); // re-add

        assert_eq!(recent.len(), 3);
        assert_eq!(recent.entries()[0], PathBuf::from("/a"));
        assert_eq!(recent.entries()[1], PathBuf::from("/c"));
        assert_eq!(recent.entries()[2], PathBuf::from("/b"));
    }

    #[test]
    fn max_entries_cap() {
        let mut recent = RecentFiles::with_max(3);
        recent.add("/a");
        recent.add("/b");
        recent.add("/c");
        recent.add("/d"); // should evict /a

        assert_eq!(recent.len(), 3);
        assert_eq!(recent.entries()[0], PathBuf::from("/d"));
        assert_eq!(recent.entries()[2], PathBuf::from("/b"));
    }

    #[test]
    fn remove() {
        let mut recent = RecentFiles::new();
        recent.add("/a");
        recent.add("/b");
        recent.add("/c");
        recent.remove(Path::new("/b"));

        assert_eq!(recent.len(), 2);
        assert_eq!(
            recent.entries(),
            &[PathBuf::from("/c"), PathBuf::from("/a")]
        );
    }

    #[test]
    fn clear() {
        let mut recent = RecentFiles::new();
        recent.add("/a");
        recent.add("/b");
        recent.clear();
        assert!(recent.is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let mut recent = RecentFiles::with_max(5);
        recent.add("/x");
        recent.add("/y");

        let json = serde_json::to_string(&recent).unwrap();
        let loaded: RecentFiles = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.most_recent(), Some(Path::new("/y")));
        assert_eq!(loaded.max_entries(), 5);
    }

    #[test]
    fn default_is_empty() {
        let recent = RecentFiles::default();
        assert!(recent.is_empty());
    }

    #[test]
    fn with_max_zero() {
        let mut recent = RecentFiles::with_max(0);
        recent.add("/a");
        assert!(recent.is_empty());
    }

    #[test]
    fn with_max_large() {
        let recent = RecentFiles::with_max(10_000);
        assert_eq!(recent.max_entries(), 10_000);
        assert!(recent.is_empty());
    }

    #[test]
    fn prune_missing_removes_nonexistent() {
        let mut recent = RecentFiles::new();
        recent.add("/nonexistent/path/that/does/not/exist");
        recent.add(std::env::temp_dir()); // this exists
        recent.prune_missing();

        assert_eq!(recent.len(), 1);
        assert_eq!(recent.entries()[0], std::env::temp_dir());
    }
}
