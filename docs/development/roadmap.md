# Muharrir Roadmap

> Shared editor primitives for AGNOS creative applications.

## Next — Interaction Primitives

- [ ] `clipboard` module — internal typed clipboard with undo integration
- [ ] `keybinding` module — context-sensitive shortcut registry, rebindable, conflict detection
- [ ] `progress` module — cancelable progress handles for long operations

## Future — Document & Polish

- [ ] `document` module — metadata (id, timestamps, name), autosave with recovery
- [ ] Hierarchy improvements — `find_by_id()`, `is_leaf()`, `node_count()`, tree mutation (reparent, insert, remove)
- [ ] Inspector improvements — `get()`, `update_value()` accessors, typed property values
- [ ] Command improvements — `undo_description()`, `redo_description()`, command merging
- [ ] Expression improvements — variable binding `eval_with_vars()`, compiled expressions

## Future — Wishlist

- [ ] `tool` module — abstract tool trait (activate/deactivate/input handling); likely needs a dedicated crate
- [ ] `plugin` module — plugin registry/loading; likely needs a dedicated crate

## Future — Consumer Migration

- [ ] Migration guides for shruti, tazama, rasa
- [ ] API freeze
- [ ] Full documentation with domain-specific examples

## Moved to ranga

Visual primitives (PixelBuffer, Color, Filter, Geometry, Spatial Selection) belong in ranga, not muharrir.
