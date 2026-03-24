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

## V0.2 — Command Pattern

- [ ] Generic `Command` trait with `apply()` / `reverse()` (inspired by shruti's UndoManager)
- [ ] Compound commands (group multiple edits into single undo entry)
- [ ] Boxed heavy variants pattern for cache-efficient command enums
- [ ] History integration with command trait

## V0.3 — Notifications

- [ ] Toast/notification system with severity levels and auto-expiry
- [ ] Notification history for console/log panel display

## V0.4 — Selection & State

- [ ] Generic selection tracker (single, multi, range)
- [ ] Selection change events for history recording
- [ ] Panel visibility state management

## V1.0 — Production

- [ ] Stabilize public API
- [ ] Documentation with domain-specific examples (game, image, audio, video)
- [ ] Performance optimization pass
- [ ] Publish to crates.io
