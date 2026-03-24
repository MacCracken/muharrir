# Muharrir Roadmap

> Shared editor primitives for AGNOS creative applications.

## V0.1 — Scaffold (done, 2026-03-23)

- hierarchy module — generic tree builder with depth-first flattening
- inspector module — PropertySheet and Property types
- history module — undo/redo via libro audit chain
- expr module — math expression evaluation via abaco
- hw module — hardware detection via ai-hwaccel
- Feature-gated architecture
- Criterion benchmarks, integration tests, example

## V0.2 — Command Pattern (done, 2026-03-23)

- [x] Generic `Command` trait with `apply()` / `reverse()` (inspired by shruti's UndoManager)
- [x] Compound commands (group multiple edits into single undo entry)
- [x] Boxed heavy variants pattern for cache-efficient command enums (documented)
- [x] History integration with command trait (compositional — `CommandHistory` + `History`)
- [x] Notifications — toast system with severity levels, auto-expiry, persistent log
- [x] Selection — generic tracker (single, multi, toggle), primary item, panel visibility
- [x] API stabilization pass — #[must_use], Clone, tracing consistency

## V0.3 — Editor Lifecycle

- [ ] `dirty` module — modified state tracking with save-point integration
- [ ] `recent` module — recent files/projects list with cap and persistence
- [ ] `prefs` module — layered preferences with JSON I/O, XDG paths

## V0.4 — Interaction Primitives

- [ ] `clipboard` module — internal typed clipboard with undo integration
- [ ] `keybinding` module — context-sensitive shortcut registry, rebindable, conflict detection
- [ ] `progress` module — cancelable progress handles for long operations

## V0.5 — Document & Polish

- [ ] `document` module — metadata (id, timestamps, name), autosave with recovery
- [ ] Hierarchy improvements — `find_by_id()`, `is_leaf()`, `node_count()`
- [ ] Inspector improvements — `get()`, `update_value()` accessors
- [ ] Command improvements — `undo_description()`, `redo_description()`
- [ ] Expression improvements — variable binding `eval_with_vars()`

## V1.0 — Consumer Migration

- [ ] Migration guides for shruti, tazama, rasa
- [ ] API freeze
- [ ] Full documentation with domain-specific examples
