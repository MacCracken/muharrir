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

## V0.3 — Notifications (done, 2026-03-23)

- [x] Toast/notification system with severity levels and auto-expiry
- [x] Notification history for console/log panel display

## V0.4 — Selection & State (done, 2026-03-23)

- [x] Generic selection tracker (single, multi, toggle)
- [x] Primary item tracking for inspector display
- [x] Panel visibility state management

## V1.0 — Production (done, 2026-03-23)

- [x] Stabilize public API — #[must_use], Clone, tracing consistency pass
- [x] Performance optimization pass — 16 benchmarks tracked, no regressions
- [x] Documentation — CHANGELOG, roadmap, examples updated
