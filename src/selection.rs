//! Generic selection tracking and panel visibility state.
//!
//! Provides [`Selection<T>`] for tracking which items are selected in an editor
//! (entities, layers, tracks, clips) and [`PanelStates`] for managing panel
//! visibility toggles.
//!
//! # Selection
//!
//! Supports single-select, multi-select (ctrl-click), and clear operations.
//! Tracks a "primary" item (the most recently selected) for inspector display.
//! Items are stored in insertion order.
//!
//! # Panel State
//!
//! [`PanelStates`] is a string-keyed visibility map. Consumers register panels
//! by name and toggle/query visibility. Serializable for layout persistence.

use std::borrow::Cow;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

/// Generic selection tracker for editor items.
///
/// `T` is the item identifier type (e.g. `u64`, `Uuid`, `TrackId`).
/// Items are stored in selection order with duplicates prevented.
#[derive(Debug, Clone)]
pub struct Selection<T> {
    items: Vec<T>,
    /// Index of the primary (most recently selected) item in `items`.
    primary_idx: Option<usize>,
}

impl<T: PartialEq + Clone> Selection<T> {
    /// Create an empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            primary_idx: None,
        }
    }

    /// Replace the entire selection with a single item.
    pub fn select(&mut self, item: T) {
        self.items.clear();
        self.items.push(item);
        self.primary_idx = Some(0);
        tracing::debug!(count = 1, "selection replaced");
    }

    /// Toggle an item in the selection (add if absent, remove if present).
    ///
    /// Used for ctrl-click multi-select.
    pub fn toggle(&mut self, item: T) {
        if let Some(pos) = self.items.iter().position(|i| *i == item) {
            self.items.remove(pos);
            self.primary_idx = self.adjust_primary_after_remove(pos);
        } else {
            self.items.push(item);
            self.primary_idx = Some(self.items.len() - 1);
        }
        tracing::debug!(count = self.items.len(), "selection toggled");
    }

    /// Add an item to the selection without removing others.
    pub fn add(&mut self, item: T) {
        let pos = if let Some(existing) = self.items.iter().position(|i| *i == item) {
            existing
        } else {
            self.items.push(item);
            self.items.len() - 1
        };
        self.primary_idx = Some(pos);
        tracing::debug!(count = self.items.len(), "selection add");
    }

    /// Remove a specific item from the selection.
    pub fn remove(&mut self, item: &T) {
        if let Some(pos) = self.items.iter().position(|i| i == item) {
            self.items.remove(pos);
            self.primary_idx = self.adjust_primary_after_remove(pos);
            tracing::debug!(count = self.items.len(), "selection remove");
        }
    }

    /// Adjust primary_idx after removing an item at `removed_pos`.
    fn adjust_primary_after_remove(&self, removed_pos: usize) -> Option<usize> {
        let primary = self.primary_idx?;
        if self.items.is_empty() {
            return None;
        }
        if removed_pos == primary {
            // Primary was removed — fall back to last item
            Some(self.items.len().saturating_sub(1))
        } else if removed_pos < primary {
            // Item before primary removed — shift index down
            Some(primary - 1)
        } else {
            // Item after primary removed — no change
            Some(primary)
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.items.clear();
        self.primary_idx = None;
        tracing::debug!("selection cleared");
    }

    /// Whether an item is selected.
    #[must_use]
    #[inline]
    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }

    /// The primary (most recently selected) item, used for inspector display.
    #[must_use]
    #[inline]
    pub fn primary(&self) -> Option<&T> {
        self.primary_idx.and_then(|i| self.items.get(i))
    }

    /// All selected items in selection order.
    #[must_use]
    #[inline]
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Number of selected items.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether nothing is selected.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Whether exactly one item is selected.
    #[must_use]
    #[inline]
    pub fn is_single(&self) -> bool {
        self.items.len() == 1
    }

    /// Replace the selection with multiple items. The last item becomes primary.
    pub fn select_many(&mut self, items: impl IntoIterator<Item = T>) {
        self.items.clear();
        for item in items {
            if !self.items.contains(&item) {
                self.items.push(item);
            }
        }
        self.primary_idx = if self.items.is_empty() {
            None
        } else {
            Some(self.items.len() - 1)
        };
        tracing::debug!(count = self.items.len(), "selection set");
    }
}

impl<T: PartialEq + Clone> Default for Selection<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PanelStates
// ---------------------------------------------------------------------------

/// Panel visibility state manager.
///
/// Tracks which editor panels are visible by name. Serializable for
/// layout persistence across sessions.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PanelStates {
    panels: HashMap<Cow<'static, str>, bool>,
}

impl PanelStates {
    /// Create an empty panel state manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a panel with an initial visibility state.
    pub fn register(&mut self, name: &'static str, visible: bool) {
        self.panels.insert(Cow::Borrowed(name), visible);
        tracing::debug!(panel = name, visible, "panel registered");
    }

    /// Whether a panel is visible. Returns `false` for unregistered panels.
    #[must_use]
    #[inline]
    pub fn is_visible(&self, name: &str) -> bool {
        self.panels.get(name).copied().unwrap_or(false)
    }

    /// Set a panel's visibility. No-op for unregistered panels.
    pub fn set_visible(&mut self, name: &str, visible: bool) {
        if let Some(v) = self.panels.get_mut(name) {
            *v = visible;
            tracing::debug!(panel = name, visible, "panel visibility changed");
        } else {
            tracing::warn!(panel = name, "set_visible called on unregistered panel");
        }
    }

    /// Toggle a panel's visibility. Returns the new state.
    /// Returns `None` if the panel is not registered.
    pub fn toggle(&mut self, name: &str) -> Option<bool> {
        let v = self.panels.get_mut(name)?;
        *v = !*v;
        tracing::debug!(panel = name, visible = *v, "panel toggled");
        Some(*v)
    }

    /// All registered panel names and their visibility.
    #[must_use]
    #[inline]
    pub fn panels(&self) -> &HashMap<Cow<'static, str>, bool> {
        &self.panels
    }

    /// Number of registered panels.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Whether no panels are registered.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Show all panels.
    pub fn show_all(&mut self) {
        for v in self.panels.values_mut() {
            *v = true;
        }
        tracing::debug!("all panels shown");
    }

    /// Hide all panels.
    pub fn hide_all(&mut self) {
        for v in self.panels.values_mut() {
            *v = false;
        }
        tracing::debug!("all panels hidden");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // === Selection ===

    #[test]
    fn selection_empty() {
        let sel: Selection<u64> = Selection::new();
        assert!(sel.is_empty());
        assert_eq!(sel.len(), 0);
        assert!(sel.primary().is_none());
    }

    #[test]
    fn selection_select_single() {
        let mut sel = Selection::new();
        sel.select(42u64);
        assert_eq!(sel.len(), 1);
        assert!(sel.is_single());
        assert!(sel.contains(&42));
        assert_eq!(sel.primary(), Some(&42));
    }

    #[test]
    fn selection_select_replaces() {
        let mut sel = Selection::new();
        sel.select(1u64);
        sel.select(2);
        assert_eq!(sel.len(), 1);
        assert!(!sel.contains(&1));
        assert!(sel.contains(&2));
    }

    #[test]
    fn selection_toggle_add_remove() {
        let mut sel = Selection::new();
        sel.toggle(1u64);
        assert!(sel.contains(&1));
        sel.toggle(2);
        assert_eq!(sel.len(), 2);
        sel.toggle(1); // remove
        assert_eq!(sel.len(), 1);
        assert!(!sel.contains(&1));
        assert!(sel.contains(&2));
    }

    #[test]
    fn selection_add_no_duplicates() {
        let mut sel = Selection::new();
        sel.add(1u64);
        sel.add(1);
        sel.add(2);
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn selection_remove() {
        let mut sel = Selection::new();
        sel.select_many([1u64, 2, 3]);
        sel.remove(&2);
        assert_eq!(sel.items(), &[1, 3]);
    }

    #[test]
    fn selection_clear() {
        let mut sel = Selection::new();
        sel.select_many([1u64, 2, 3]);
        sel.clear();
        assert!(sel.is_empty());
        assert!(sel.primary().is_none());
    }

    #[test]
    fn selection_primary_tracks_last() {
        let mut sel = Selection::new();
        sel.select_many([1u64, 2, 3]);
        assert_eq!(sel.primary(), Some(&3));

        sel.toggle(2); // removes 2, primary stays at last
        assert_eq!(sel.primary(), Some(&3));

        sel.toggle(5); // adds 5, becomes primary
        assert_eq!(sel.primary(), Some(&5));
    }

    #[test]
    fn selection_select_many_deduplicates() {
        let mut sel = Selection::new();
        sel.select_many([1u64, 2, 1, 3, 2]);
        assert_eq!(sel.len(), 3);
        assert_eq!(sel.items(), &[1, 2, 3]);
    }

    #[test]
    fn selection_remove_preserves_primary() {
        let mut sel = Selection::new();
        sel.select_many([1u64, 2, 3, 4, 5]);
        // Primary is 5 (last, index 4)
        assert_eq!(sel.primary(), Some(&5));

        // Remove item before primary — primary stays at 5
        sel.remove(&2);
        assert_eq!(sel.items(), &[1, 3, 4, 5]);
        assert_eq!(sel.primary(), Some(&5));

        // Remove primary itself — falls back to last
        sel.remove(&5);
        assert_eq!(sel.items(), &[1, 3, 4]);
        assert_eq!(sel.primary(), Some(&4));
    }

    #[test]
    fn selection_toggle_off_preserves_primary() {
        let mut sel = Selection::new();
        sel.select_many([1u64, 2, 3]);
        // Primary is at index 2 (value 3)
        assert_eq!(sel.primary(), Some(&3));

        // Toggle off item before primary
        sel.toggle(1);
        assert_eq!(sel.items(), &[2, 3]);
        assert_eq!(sel.primary(), Some(&3)); // still 3, not shifted to 2
    }

    #[test]
    fn selection_remove_after_primary_no_shift() {
        let mut sel = Selection::new();
        sel.select(1u64);
        sel.add(2);
        sel.add(3);
        // Primary is at index 2 (value 3, last added)
        assert_eq!(sel.primary(), Some(&3));

        // Select item 1 as primary
        sel.add(1);
        assert_eq!(sel.primary(), Some(&1)); // index 0

        // Remove item after primary
        sel.remove(&3);
        assert_eq!(sel.primary(), Some(&1)); // unchanged
    }

    #[test]
    fn selection_default() {
        let sel: Selection<u32> = Selection::default();
        assert!(sel.is_empty());
    }

    // === PanelStates ===

    #[test]
    fn panel_states_register_and_query() {
        let mut panels = PanelStates::new();
        panels.register("inspector", true);
        panels.register("hierarchy", true);
        panels.register("viewport", false);

        assert!(panels.is_visible("inspector"));
        assert!(panels.is_visible("hierarchy"));
        assert!(!panels.is_visible("viewport"));
        assert!(!panels.is_visible("unknown")); // unregistered → false
        assert_eq!(panels.len(), 3);
    }

    #[test]
    fn panel_states_set_visible() {
        let mut panels = PanelStates::new();
        panels.register("inspector", true);
        panels.set_visible("inspector", false);
        assert!(!panels.is_visible("inspector"));
    }

    #[test]
    fn panel_states_toggle() {
        let mut panels = PanelStates::new();
        panels.register("inspector", true);

        let new_state = panels.toggle("inspector");
        assert_eq!(new_state, Some(false));
        assert!(!panels.is_visible("inspector"));

        let new_state = panels.toggle("inspector");
        assert_eq!(new_state, Some(true));
    }

    #[test]
    fn panel_states_toggle_unregistered() {
        let mut panels = PanelStates::new();
        assert_eq!(panels.toggle("nope"), None);
    }

    #[test]
    fn panel_states_show_hide_all() {
        let mut panels = PanelStates::new();
        panels.register("a", true);
        panels.register("b", false);
        panels.register("c", true);

        panels.hide_all();
        assert!(!panels.is_visible("a"));
        assert!(!panels.is_visible("b"));
        assert!(!panels.is_visible("c"));

        panels.show_all();
        assert!(panels.is_visible("a"));
        assert!(panels.is_visible("b"));
        assert!(panels.is_visible("c"));
    }

    #[test]
    fn panel_states_serde_roundtrip() {
        let mut panels = PanelStates::new();
        panels.register("inspector", true);
        panels.register("hierarchy", false);

        let json = serde_json::to_string(&panels).unwrap();
        let deserialized: PanelStates = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_visible("inspector"));
        assert!(!deserialized.is_visible("hierarchy"));
    }

    #[test]
    fn panel_states_default_empty() {
        let panels = PanelStates::default();
        assert!(panels.is_empty());
    }
}
