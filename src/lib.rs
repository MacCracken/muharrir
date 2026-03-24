//! Muharrir — shared editor primitives for AGNOS creative applications.
//!
//! Provides reusable building blocks for editor UIs: undo/redo history,
//! expression evaluation, hardware detection, hierarchy trees, and property
//! inspection. Used by salai (game editor), rasa (image editor),
//! tazama (video editor), and shruti (audio DAW).
//!
//! # Feature Flags
//!
//! | Feature | Default | Dependencies | Description |
//! |---------|---------|-------------|-------------|
//! | `history` | yes | libro, serde_json | Undo/redo with tamper-evident audit chain |
//! | `expr` | yes | abaco | Math expression evaluation for property fields |
//! | `hw` | yes | ai-hwaccel | Hardware detection and quality tiers |
//! | `hierarchy` | yes | — | Generic parent-child tree building |
//! | `inspector` | yes | — | Property sheet for editor panels |
//! | `command` | yes | — | Generic command pattern with undo/redo stacks |
//! | `personality` | no | bhava | NPC personality/emotion editing |
//! | `full` | no | all above | Everything enabled |

pub mod error;

#[cfg(feature = "hierarchy")]
pub mod hierarchy;

#[cfg(feature = "history")]
pub mod history;

#[cfg(feature = "hw")]
pub mod hw;

#[cfg(feature = "inspector")]
pub mod inspector;

#[cfg(feature = "expr")]
pub mod expr;

#[cfg(feature = "command")]
pub mod command;

// Re-exports
pub use error::{Error, Result};

#[cfg(feature = "hierarchy")]
pub use hierarchy::{FlatEntry, HierarchyNode, NodeId, build_hierarchy, flatten};

#[cfg(feature = "history")]
pub use history::{Action, History};

#[cfg(feature = "hw")]
pub use hw::{HardwareProfile, QualityTier};

#[cfg(feature = "inspector")]
pub use inspector::{Property, PropertySheet};

#[cfg(feature = "expr")]
pub use expr::{ExprError, eval_f64, eval_or, eval_or_parse};

#[cfg(feature = "command")]
pub use command::{Command, CommandHistory, CompoundCommand};
