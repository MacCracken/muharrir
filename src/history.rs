//! Undo/redo history backed by [`libro::AuditChain`].
//!
//! Combines shruti's apply/reverse command pattern with libro's tamper-evident
//! audit chain. Each editor action is recorded as a verifiable entry with
//! cursor-based undo/redo navigation.

#[cfg(feature = "history")]
use libro::{AuditChain, AuditEntry, EventSeverity};
#[cfg(feature = "history")]
use serde_json::Value;

/// An editor action recorded in the history.
#[cfg(feature = "history")]
#[derive(Debug, Clone)]
pub struct Action {
    /// What kind of action (e.g. "set_position", "add_layer", "set_gain").
    pub kind: String,
    /// JSON payload with the action details (before/after state for reversal).
    pub details: Value,
}

#[cfg(feature = "history")]
impl Action {
    /// Create a new action.
    #[must_use]
    pub fn new(kind: impl Into<String>, details: Value) -> Self {
        Self {
            kind: kind.into(),
            details,
        }
    }
}

/// Undo/redo history for any editor.
#[cfg(feature = "history")]
pub struct History {
    chain: AuditChain,
    /// Current position in the history (points past the last applied action).
    cursor: usize,
}

#[cfg(feature = "history")]
impl History {
    /// Create a new empty history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            chain: AuditChain::new(),
            cursor: 0,
        }
    }

    /// Record a new action. Invalidates the redo stack.
    pub fn record(&mut self, source: &str, action: Action) {
        self.cursor = self.chain.len();
        self.chain
            .append(EventSeverity::Info, source, &action.kind, action.details);
        self.cursor = self.chain.len();
        tracing::debug!(source, kind = action.kind, cursor = self.cursor, "action recorded");
    }

    /// Whether there are actions that can be undone.
    #[must_use]
    #[inline]
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    /// Whether there are actions that can be redone.
    #[must_use]
    #[inline]
    pub fn can_redo(&self) -> bool {
        self.cursor < self.chain.len()
    }

    /// Move the cursor back one step. Returns the action to undo.
    pub fn undo(&mut self) -> Option<&AuditEntry> {
        if !self.can_undo() {
            return None;
        }
        self.cursor -= 1;
        tracing::debug!(cursor = self.cursor, "undo");
        self.chain.entries().get(self.cursor)
    }

    /// Move the cursor forward one step. Returns the action to redo.
    pub fn redo(&mut self) -> Option<&AuditEntry> {
        if !self.can_redo() {
            return None;
        }
        let entry = self.chain.entries().get(self.cursor);
        self.cursor += 1;
        tracing::debug!(cursor = self.cursor, "redo");
        entry
    }

    /// Number of total recorded actions.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Whether the history is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    /// Current cursor position (number of applied actions).
    #[must_use]
    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Verify the integrity of the entire history chain.
    #[must_use]
    pub fn verify(&self) -> bool {
        self.chain.verify().is_ok()
    }

    /// Get all entries in the chain.
    #[must_use]
    pub fn entries(&self) -> &[AuditEntry] {
        self.chain.entries()
    }

    /// Get entries up to the current cursor (the "applied" history).
    #[must_use]
    pub fn applied_entries(&self) -> &[AuditEntry] {
        &self.chain.entries()[..self.cursor]
    }

    /// Get a page of entries for display.
    #[must_use]
    pub fn page(&self, offset: usize, limit: usize) -> &[AuditEntry] {
        self.chain.page(offset, limit)
    }
}

#[cfg(feature = "history")]
impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "history"))]
mod tests {
    use super::*;
    use serde_json::json;

    fn action(kind: &str) -> Action {
        Action::new(kind, json!({"test": true}))
    }

    #[test]
    fn new_history_is_empty() {
        let h = History::new();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
        assert_eq!(h.cursor(), 0);
        assert!(!h.can_undo());
        assert!(!h.can_redo());
    }

    #[test]
    fn record_increments_cursor() {
        let mut h = History::new();
        h.record("test", action("do_something"));
        assert_eq!(h.len(), 1);
        assert_eq!(h.cursor(), 1);
        assert!(h.can_undo());
        assert!(!h.can_redo());
    }

    #[test]
    fn undo_moves_cursor_back() {
        let mut h = History::new();
        h.record("test", action("a"));
        h.record("test", action("b"));

        let entry = h.undo().unwrap();
        assert_eq!(entry.action(), "b");
        assert_eq!(h.cursor(), 1);
        assert!(h.can_undo());
        assert!(h.can_redo());
    }

    #[test]
    fn redo_moves_cursor_forward() {
        let mut h = History::new();
        h.record("test", action("a"));
        h.undo();

        let entry = h.redo().unwrap();
        assert_eq!(entry.action(), "a");
        assert_eq!(h.cursor(), 1);
        assert!(!h.can_redo());
    }

    #[test]
    fn undo_at_start_returns_none() {
        let mut h = History::new();
        assert!(h.undo().is_none());
    }

    #[test]
    fn redo_at_end_returns_none() {
        let mut h = History::new();
        h.record("test", action("a"));
        assert!(h.redo().is_none());
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut h = History::new();
        h.record("test", action("a"));
        h.record("test", action("b"));
        h.record("test", action("c"));

        h.undo();
        h.undo();
        assert_eq!(h.cursor(), 1);

        h.redo();
        h.redo();
        assert_eq!(h.cursor(), 3);
        assert!(!h.can_redo());
    }

    #[test]
    fn verify_intact_chain() {
        let mut h = History::new();
        h.record("editor", action("move"));
        h.record("editor", action("rotate"));
        assert!(h.verify());
    }

    #[test]
    fn applied_entries_tracks_cursor() {
        let mut h = History::new();
        h.record("test", action("a"));
        h.record("test", action("b"));
        h.record("test", action("c"));

        assert_eq!(h.applied_entries().len(), 3);
        h.undo();
        assert_eq!(h.applied_entries().len(), 2);
    }

    #[test]
    fn page_returns_slice() {
        let mut h = History::new();
        for i in 0..10 {
            h.record("test", action(&format!("action_{i}")));
        }
        let page = h.page(5, 3);
        assert_eq!(page.len(), 3);
    }

    #[test]
    fn entries_preserves_details() {
        let mut h = History::new();
        let details = json!({"entity": 42, "before": [0, 0, 0], "after": [1, 2, 3]});
        h.record("inspector", Action::new("set_position", details.clone()));
        assert_eq!(h.entries()[0].details(), &details);
    }

    #[test]
    fn default_trait() {
        let h = History::default();
        assert!(h.is_empty());
    }
}
