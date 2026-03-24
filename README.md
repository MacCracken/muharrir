# Muharrir

> **Muharrir** (Arabic: محرر — editor/author) — Shared editor primitives for AGNOS creative applications.

Reusable building blocks for editor UIs: undo/redo history, expression evaluation, hardware detection, hierarchy trees, and property inspection.

## Consumers

| App | Domain | Uses |
|-----|--------|------|
| [salai](https://github.com/MacCracken/salai) | Game editor | hierarchy, inspector, history, expr, hw |
| [rasa](https://github.com/MacCracken/rasa) | Image editor | history, inspector, hierarchy, hw |
| [tazama](https://github.com/MacCracken/tazama) | Video editor | history, hierarchy, hw, expr |
| [shruti](https://github.com/MacCracken/shruti) | Audio DAW | history, inspector, hierarchy, expr |

## Feature Flags

| Feature | Default | Dependencies | Description |
|---------|---------|-------------|-------------|
| `history` | yes | libro, serde_json | Undo/redo with tamper-evident audit chain |
| `expr` | yes | abaco | Math expression evaluation for property fields |
| `hw` | yes | ai-hwaccel | Hardware detection and quality tiers |
| `hierarchy` | yes | — | Generic parent-child tree building |
| `inspector` | yes | — | Property sheet for editor panels |
| `personality` | no | bhava | NPC personality/emotion editing |
| `full` | no | all above | Everything enabled |

## Quick Start

```rust
use muharrir::{PropertySheet, Property, build_hierarchy, flatten};

// Build a hierarchy tree
let tree = build_hierarchy(
    &[1, 2, 3],
    |id| if id > 1 { Some(1) } else { None },
    |id| format!("Node {id}"),
);
let flat = flatten(&tree);

// Build a property sheet
let mut sheet = PropertySheet::new();
sheet.push(Property::new("Transform", "position", "(1, 2, 3)"));
```

## License

GPL-3.0
