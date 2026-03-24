# Changelog

## [0.1.0] - 2026-03-23

### Added
- **hierarchy** module — generic parent-child tree builder with depth-first flattening.
- **inspector** module — `PropertySheet` and `Property` types for editor panels.
- **history** module — undo/redo system backed by `libro` audit chain with tamper-evident verification.
- **expr** module — math expression evaluator via `abaco` (arithmetic, trig, constants).
- **hw** module — hardware capability detection via `ai-hwaccel` with quality tier mapping.
- **error** module — unified error types with `thiserror`.
- Feature-gated architecture — consumers pull only what they need.
- Criterion benchmarks for all modules.
- Integration tests covering cross-module workflows.
- Example: `basic.rs` demonstrating all modules.
