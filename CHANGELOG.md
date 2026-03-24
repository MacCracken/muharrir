# Changelog

## [0.23.3] - 2026-03-23

### Added
- **command** module — generic `Command` trait with `apply()`/`reverse()`, `CompoundCommand` for grouped edits with rollback, `CommandHistory` with VecDeque/Vec undo/redo stacks, max-depth eviction, and command preservation on error. Blanket impl for `Box<dyn Command>` trait objects.
- **notification** module — `Toast` with severity-based auto-expiry and progress tracking, `Toasts` manager with GC, `Notification` persistent log entries, `NotificationLog` with capped VecDeque and severity/source filtering.
- **selection** module — generic `Selection<T>` tracker with select/toggle/add/remove/primary, `PanelStates` for string-keyed panel visibility with serde persistence.
- **dirty** module — `DirtyState` generation-based modified tracking with save-point.
- **recent** module — `RecentFiles` capped MRU list with dedup, prune, serde.
- **prefs** module — `PrefsStore` generic JSON load/save with directory creation, Unix 0600 permissions, `config_dir()` XDG resolver.
- `PropertySheet::with_capacity()` for pre-allocation.
- Serde derives on `QualityTier`, `HardwareProfile`, `FlatEntry`, `PropertySheet` (serialize-only).
- Example: `command_usage.rs` demonstrating trait objects, compounds, and audit trail composition.

### Changed
- `Action::kind` changed from `String` to `Cow<'static, str>` — zero-allocation for static kind strings.
- `Action::new()` now takes `&'static str`; `Action::with_kind()` added for dynamic strings.
- `build_hierarchy()` refactored from O(N²) to O(N) via pre-built children map.
- Self-referencing nodes now treated as roots in hierarchy builder.
- `classify_quality()` uses floating-point division (was integer truncation).
- Thread-local `Evaluator` in expr module avoids per-call initialization.
- `CommandHistory::undo()`/`redo()` preserve commands on error (not lost).
- `CompoundCommand` and `CommandHistory` now derive `Clone` (when `C: Clone`).

### Fixed
- `deny.toml` updated for cargo-deny 0.19 format.
- SPDX license identifier updated from deprecated `GPL-3.0` to `GPL-3.0-only`.
- `applied_entries()` defensive bounds check prevents panic on invariant violation.
- `Selection::remove()`/`toggle()` correctly adjust primary index after removal.
- Zero-duration `Toast::progress()` returns 1.0 instead of NaN.

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
