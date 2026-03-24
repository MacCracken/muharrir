//! Modified/dirty state tracking for editor documents.
//!
//! Tracks whether a document has unsaved changes relative to its last save
//! point. Integrates with [`super::command::CommandHistory`] by comparing
//! the current command count against the save-point snapshot.

/// Tracks whether a document has been modified since last save.
///
/// Uses a generation counter that increments on every change. The save-point
/// records the generation at the last save, so `is_dirty()` is a simple
/// comparison — O(1), no scanning.
#[derive(Debug, Clone)]
pub struct DirtyState {
    generation: u64,
    save_point: u64,
}

impl DirtyState {
    /// Create a new clean state (not dirty).
    #[must_use]
    pub fn new() -> Self {
        Self {
            generation: 0,
            save_point: 0,
        }
    }

    /// Mark as modified. Call after any content change.
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.generation += 1;
        tracing::trace!(generation = self.generation, "marked dirty");
    }

    /// Mark as clean (saved). Call after successful save.
    #[inline]
    pub fn mark_clean(&mut self) {
        self.save_point = self.generation;
        tracing::debug!(save_point = self.save_point, "marked clean");
    }

    /// Whether the document has unsaved changes.
    #[must_use]
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.generation != self.save_point
    }

    /// Whether the document is clean (no unsaved changes).
    #[must_use]
    #[inline]
    pub fn is_clean(&self) -> bool {
        !self.is_dirty()
    }

    /// Current generation counter.
    #[must_use]
    #[inline]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Generation at last save point.
    #[must_use]
    #[inline]
    pub fn save_point(&self) -> u64 {
        self.save_point
    }

    /// Number of changes since last save.
    #[must_use]
    #[inline]
    pub fn changes_since_save(&self) -> u64 {
        self.generation.saturating_sub(self.save_point)
    }

    /// Reset to a completely clean initial state.
    pub fn reset(&mut self) {
        self.generation = 0;
        self.save_point = 0;
        tracing::debug!("dirty state reset");
    }
}

impl Default for DirtyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_clean() {
        let state = DirtyState::new();
        assert!(state.is_clean());
        assert!(!state.is_dirty());
        assert_eq!(state.changes_since_save(), 0);
    }

    #[test]
    fn mark_dirty() {
        let mut state = DirtyState::new();
        state.mark_dirty();
        assert!(state.is_dirty());
        assert_eq!(state.generation(), 1);
        assert_eq!(state.changes_since_save(), 1);
    }

    #[test]
    fn mark_clean_after_dirty() {
        let mut state = DirtyState::new();
        state.mark_dirty();
        state.mark_dirty();
        state.mark_clean();
        assert!(state.is_clean());
        assert_eq!(state.changes_since_save(), 0);
    }

    #[test]
    fn dirty_after_save_then_edit() {
        let mut state = DirtyState::new();
        state.mark_dirty();
        state.mark_clean();
        assert!(state.is_clean());

        state.mark_dirty();
        assert!(state.is_dirty());
        assert_eq!(state.changes_since_save(), 1);
    }

    #[test]
    fn multiple_edits_between_saves() {
        let mut state = DirtyState::new();
        for _ in 0..10 {
            state.mark_dirty();
        }
        assert_eq!(state.changes_since_save(), 10);

        state.mark_clean();
        assert_eq!(state.changes_since_save(), 0);
    }

    #[test]
    fn reset() {
        let mut state = DirtyState::new();
        state.mark_dirty();
        state.mark_dirty();
        state.mark_clean();
        state.mark_dirty();

        state.reset();
        assert!(state.is_clean());
        assert_eq!(state.generation(), 0);
        assert_eq!(state.save_point(), 0);
    }

    #[test]
    fn default_is_clean() {
        let state = DirtyState::default();
        assert!(state.is_clean());
    }
}
