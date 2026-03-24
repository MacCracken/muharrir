# Muharrir Architecture

> Shared editor primitives extracted from salai, informed by rasa, tazama, and shruti patterns.

## Module Map

```
muharrir
├── hierarchy   — generic tree builder (entity trees, layer stacks, track lists)
├── inspector   — property sheet pattern (component info, layer props, track params)
├── history     — undo/redo via libro audit chain (tamper-evident, cursor-based)
├── expr        — expression evaluation via abaco (math in property fields)
├── hw          — hardware detection via ai-hwaccel (quality tiers, GPU info)
└── error       — unified error types
```

## Data Flow

```
User Action → History::record() → libro::AuditChain::append()
                                        ↓
                                   SHA-256 hash chain
                                        ↓
User Undo   → History::undo()  → cursor moves back, returns entry
User Redo   → History::redo()  → cursor moves forward, returns entry
```

## Consumer Integration

Each consumer (salai, rasa, tazama, shruti) depends on muharrir with the features they need:

```toml
# Game editor
muharrir = { version = "0.1", features = ["full"] }

# Image editor (no personality needed)
muharrir = { version = "0.1" }  # defaults are sufficient

# Audio DAW (already has its own undo, but wants expr + inspector)
muharrir = { version = "0.1", default-features = false, features = ["expr", "inspector", "hierarchy"] }
```
