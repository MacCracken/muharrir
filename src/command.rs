//! Generic command pattern for reversible editor operations.
//!
//! Provides the [`Command`] trait, [`CompoundCommand`] for grouped edits, and
//! [`CommandHistory`] for undo/redo stack management. Extracted from patterns
//! shared across shruti, tazama, and rasa.
//!
//! # Design
//!
//! Commands use `&mut self` on both [`Command::apply`] and [`Command::reverse`],
//! supporting lazy state capture during apply (a command can fill in undo data
//! when executed rather than at construction time). Consumers who don't need
//! mutation simply don't mutate `self`.
//!
//! [`CommandHistory`] is deliberately independent of the libro-backed [`super::history::History`]
//! audit chain. Consumers compose both at their level when they need tamper-evident
//! logging alongside command execution.
//!
//! # Boxed heavy variants
//!
//! When implementing command enums, `Box` large variant fields so lightweight
//! variants (toggles, scalar changes) don't pay the size cost of the largest
//! variant. See shruti's `EditCommand` for the canonical example.

use std::borrow::Cow;
use std::collections::VecDeque;

/// A reversible editor command.
///
/// Consumers define domain-specific command enums and implement this trait.
/// The trait intentionally requires no `Serialize`, `Send`, or `Clone` bounds —
/// those are layered on by consumers as needed.
pub trait Command: std::fmt::Debug {
    /// The state this command operates on (e.g. a session, timeline, canvas).
    type Target;
    /// Error type. Use [`std::convert::Infallible`] for infallible commands.
    type Error: std::fmt::Debug;

    /// Apply the command to the target.
    ///
    /// May mutate `self` to capture undo state (e.g. store removed data for
    /// later restoration in [`reverse`](Command::reverse)).
    fn apply(&mut self, target: &mut Self::Target) -> Result<(), Self::Error>;

    /// Reverse the command, restoring the target to its prior state.
    fn reverse(&mut self, target: &mut Self::Target) -> Result<(), Self::Error>;

    /// Human-readable description for undo/redo menu display.
    #[must_use]
    fn description(&self) -> &str;
}

// Blanket impl for boxed trait objects, enabling `CommandHistory<Box<dyn Command<...>>>`.
impl<T, E> Command for Box<dyn Command<Target = T, Error = E>>
where
    E: std::fmt::Debug,
{
    type Target = T;
    type Error = E;

    fn apply(&mut self, target: &mut T) -> Result<(), E> {
        (**self).apply(target)
    }

    fn reverse(&mut self, target: &mut T) -> Result<(), E> {
        (**self).reverse(target)
    }

    fn description(&self) -> &str {
        (**self).description()
    }
}

// ---------------------------------------------------------------------------
// CompoundCommand
// ---------------------------------------------------------------------------

/// A group of commands executed as a single undo/redo unit.
///
/// Commands are applied in order and reversed in reverse order (matching
/// shruti's `Compound` variant behaviour). If a sub-command fails during
/// apply, all previously-applied sub-commands are rolled back (best-effort).
#[derive(Debug)]
pub struct CompoundCommand<C> {
    commands: Vec<C>,
    description: Cow<'static, str>,
}

impl<C> CompoundCommand<C> {
    /// Create a compound command with a static description.
    #[must_use]
    #[inline]
    pub fn new(description: &'static str, commands: Vec<C>) -> Self {
        Self {
            commands,
            description: Cow::Borrowed(description),
        }
    }

    /// Create a compound command with a dynamic description.
    #[must_use]
    #[inline]
    pub fn with_description(description: String, commands: Vec<C>) -> Self {
        Self {
            commands,
            description: Cow::Owned(description),
        }
    }

    /// Number of sub-commands.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether this compound command has no sub-commands.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Access the sub-commands.
    #[must_use]
    #[inline]
    pub fn commands(&self) -> &[C] {
        &self.commands
    }
}

impl<C: Command> Command for CompoundCommand<C> {
    type Target = C::Target;
    type Error = C::Error;

    fn apply(&mut self, target: &mut Self::Target) -> Result<(), Self::Error> {
        for (i, cmd) in self.commands.iter_mut().enumerate() {
            if let Err(e) = cmd.apply(target) {
                tracing::error!(
                    failed_at = i,
                    total = self.commands.len(),
                    "compound apply failed, rolling back"
                );
                for rollback in self.commands[..i].iter_mut().rev() {
                    if let Err(rollback_err) = rollback.reverse(target) {
                        tracing::error!(?rollback_err, "rollback failed during compound apply");
                    }
                }
                return Err(e);
            }
        }
        tracing::debug!(count = self.commands.len(), desc = %self.description, "compound applied");
        Ok(())
    }

    fn reverse(&mut self, target: &mut Self::Target) -> Result<(), Self::Error> {
        for cmd in self.commands.iter_mut().rev() {
            cmd.reverse(target)?;
        }
        tracing::debug!(count = self.commands.len(), desc = %self.description, "compound reversed");
        Ok(())
    }

    #[inline]
    fn description(&self) -> &str {
        &self.description
    }
}

// ---------------------------------------------------------------------------
// CommandHistory
// ---------------------------------------------------------------------------

/// Default maximum undo depth.
const DEFAULT_MAX_DEPTH: usize = 256;

/// Undo/redo stack for commands.
///
/// Uses [`VecDeque`] for the undo stack (O(1) eviction at `max_depth`) and
/// [`Vec`] for the redo stack (only needs push/pop/clear). This matches the
/// pattern used in shruti's `UndoManager` and rasa's `History`.
#[derive(Debug)]
pub struct CommandHistory<C> {
    undo_stack: VecDeque<C>,
    redo_stack: Vec<C>,
    max_depth: usize,
}

impl<C> CommandHistory<C> {
    /// Create a new command history with default max depth (256).
    #[must_use]
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// Create a new command history with a custom max depth.
    #[must_use]
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_depth.min(1024)),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    /// Whether there are commands that can be undone.
    #[must_use]
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there are commands that can be redone.
    #[must_use]
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Number of commands in the undo stack.
    #[must_use]
    #[inline]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of commands in the redo stack.
    #[must_use]
    #[inline]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Maximum undo depth.
    #[must_use]
    #[inline]
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Clear all history (both undo and redo stacks).
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        tracing::debug!("command history cleared");
    }
}

impl<C: Command> CommandHistory<C> {
    /// Execute a command: apply it to the target, then push onto the undo stack.
    ///
    /// Clears the redo stack (branching). Evicts the oldest command if at max depth.
    pub fn execute(&mut self, mut cmd: C, target: &mut C::Target) -> Result<(), C::Error> {
        cmd.apply(target)?;
        tracing::debug!(desc = cmd.description(), "command executed");
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(cmd);
        Ok(())
    }

    /// Undo the last command. Returns `Ok(true)` if a command was undone,
    /// `Ok(false)` if the undo stack was empty.
    ///
    /// On error the command is restored to the undo stack (not lost).
    pub fn undo(&mut self, target: &mut C::Target) -> Result<bool, C::Error> {
        let Some(mut cmd) = self.undo_stack.pop_back() else {
            return Ok(false);
        };
        tracing::debug!(desc = cmd.description(), "undoing");
        if let Err(e) = cmd.reverse(target) {
            self.undo_stack.push_back(cmd);
            return Err(e);
        }
        self.redo_stack.push(cmd);
        Ok(true)
    }

    /// Redo the last undone command. Returns `Ok(true)` if a command was redone,
    /// `Ok(false)` if the redo stack was empty.
    ///
    /// On error the command is restored to the redo stack (not lost).
    pub fn redo(&mut self, target: &mut C::Target) -> Result<bool, C::Error> {
        let Some(mut cmd) = self.redo_stack.pop() else {
            return Ok(false);
        };
        tracing::debug!(desc = cmd.description(), "redoing");
        if let Err(e) = cmd.apply(target) {
            self.redo_stack.push(cmd);
            return Err(e);
        }
        self.undo_stack.push_back(cmd);
        Ok(true)
    }
}

impl<C> Default for CommandHistory<C> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    // -- Test command: push/pop on a Vec<i32> --

    #[derive(Debug)]
    struct PushCmd {
        value: i32,
    }

    impl Command for PushCmd {
        type Target = Vec<i32>;
        type Error = Infallible;

        fn apply(&mut self, target: &mut Vec<i32>) -> Result<(), Infallible> {
            target.push(self.value);
            Ok(())
        }

        fn reverse(&mut self, target: &mut Vec<i32>) -> Result<(), Infallible> {
            target.pop();
            Ok(())
        }

        fn description(&self) -> &str {
            "push"
        }
    }

    // -- Test command: lazy state capture --

    #[derive(Debug)]
    struct RemoveLastCmd {
        removed: Option<i32>,
    }

    impl RemoveLastCmd {
        fn new() -> Self {
            Self { removed: None }
        }
    }

    impl Command for RemoveLastCmd {
        type Target = Vec<i32>;
        type Error = Infallible;

        fn apply(&mut self, target: &mut Vec<i32>) -> Result<(), Infallible> {
            self.removed = target.pop();
            Ok(())
        }

        fn reverse(&mut self, target: &mut Vec<i32>) -> Result<(), Infallible> {
            if let Some(v) = self.removed {
                target.push(v);
            }
            Ok(())
        }

        fn description(&self) -> &str {
            "remove last"
        }
    }

    // -- Test command: fallible --

    #[derive(Debug)]
    struct FailOnThreshold {
        value: i32,
        threshold: usize,
    }

    impl Command for FailOnThreshold {
        type Target = Vec<i32>;
        type Error = String;

        fn apply(&mut self, target: &mut Vec<i32>) -> Result<(), String> {
            if target.len() >= self.threshold {
                return Err("threshold reached".into());
            }
            target.push(self.value);
            Ok(())
        }

        fn reverse(&mut self, target: &mut Vec<i32>) -> Result<(), String> {
            target.pop();
            Ok(())
        }

        fn description(&self) -> &str {
            "fail on threshold"
        }
    }

    // === Command trait tests ===

    #[test]
    fn apply_and_reverse() {
        let mut target = vec![];
        let mut cmd = PushCmd { value: 42 };
        cmd.apply(&mut target).unwrap();
        assert_eq!(target, vec![42]);
        cmd.reverse(&mut target).unwrap();
        assert!(target.is_empty());
    }

    #[test]
    fn lazy_state_capture() {
        let mut target = vec![10, 20, 30];
        let mut cmd = RemoveLastCmd::new();
        assert!(cmd.removed.is_none());

        cmd.apply(&mut target).unwrap();
        assert_eq!(target, vec![10, 20]);
        assert_eq!(cmd.removed, Some(30));

        cmd.reverse(&mut target).unwrap();
        assert_eq!(target, vec![10, 20, 30]);
    }

    #[test]
    fn infallible_commands_compile() {
        let mut target = vec![];
        let mut cmd = PushCmd { value: 1 };
        let result: Result<(), Infallible> = cmd.apply(&mut target);
        assert!(result.is_ok());
    }

    // === CompoundCommand tests ===

    #[test]
    fn compound_apply_in_order() {
        let mut target = vec![];
        let mut compound = CompoundCommand::new(
            "push three",
            vec![
                PushCmd { value: 1 },
                PushCmd { value: 2 },
                PushCmd { value: 3 },
            ],
        );
        compound.apply(&mut target).unwrap();
        assert_eq!(target, vec![1, 2, 3]);
    }

    #[test]
    fn compound_reverse_in_reverse_order() {
        let mut target = vec![];
        let mut compound = CompoundCommand::new(
            "push three",
            vec![
                PushCmd { value: 1 },
                PushCmd { value: 2 },
                PushCmd { value: 3 },
            ],
        );
        compound.apply(&mut target).unwrap();
        compound.reverse(&mut target).unwrap();
        assert!(target.is_empty());
    }

    #[test]
    fn compound_partial_failure_rollback() {
        let mut target = vec![];
        let mut compound = CompoundCommand::new(
            "fail on second",
            vec![
                FailOnThreshold {
                    value: 1,
                    threshold: 5,
                },
                FailOnThreshold {
                    value: 2,
                    threshold: 1, // fails when len >= 1
                },
                FailOnThreshold {
                    value: 3,
                    threshold: 5,
                },
            ],
        );
        let result = compound.apply(&mut target);
        assert!(result.is_err());
        // First command was rolled back
        assert!(target.is_empty());
    }

    #[test]
    fn compound_empty() {
        let mut target: Vec<i32> = vec![1, 2, 3];
        let mut compound: CompoundCommand<PushCmd> = CompoundCommand::new("empty", vec![]);
        compound.apply(&mut target).unwrap();
        assert_eq!(target, vec![1, 2, 3]);
        compound.reverse(&mut target).unwrap();
        assert_eq!(target, vec![1, 2, 3]);
    }

    #[test]
    fn compound_accessors() {
        let compound =
            CompoundCommand::new("test", vec![PushCmd { value: 1 }, PushCmd { value: 2 }]);
        assert_eq!(compound.len(), 2);
        assert!(!compound.is_empty());
        assert_eq!(compound.commands().len(), 2);
        assert_eq!(compound.description(), "test");
    }

    #[test]
    fn compound_with_dynamic_description() {
        let compound =
            CompoundCommand::with_description("dynamic".to_string(), vec![PushCmd { value: 1 }]);
        assert_eq!(compound.description(), "dynamic");
    }

    // === CommandHistory tests ===

    #[test]
    fn history_execute() {
        let mut target = vec![];
        let mut history = CommandHistory::new();

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.execute(PushCmd { value: 2 }, &mut target).unwrap();
        history.execute(PushCmd { value: 3 }, &mut target).unwrap();

        assert_eq!(target, vec![1, 2, 3]);
        assert_eq!(history.undo_count(), 3);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn history_undo() {
        let mut target = vec![];
        let mut history = CommandHistory::new();

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.execute(PushCmd { value: 2 }, &mut target).unwrap();

        assert!(history.undo(&mut target).unwrap());
        assert_eq!(target, vec![1]);
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 1);
    }

    #[test]
    fn history_redo() {
        let mut target = vec![];
        let mut history = CommandHistory::new();

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.undo(&mut target).unwrap();
        assert!(target.is_empty());

        assert!(history.redo(&mut target).unwrap());
        assert_eq!(target, vec![1]);
    }

    #[test]
    fn history_undo_redo_roundtrip() {
        let mut target = vec![];
        let mut history = CommandHistory::new();

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.execute(PushCmd { value: 2 }, &mut target).unwrap();
        history.execute(PushCmd { value: 3 }, &mut target).unwrap();

        history.undo(&mut target).unwrap();
        history.undo(&mut target).unwrap();
        assert_eq!(target, vec![1]);

        history.redo(&mut target).unwrap();
        history.redo(&mut target).unwrap();
        assert_eq!(target, vec![1, 2, 3]);
    }

    #[test]
    fn history_undo_empty() {
        let mut target: Vec<i32> = vec![];
        let mut history: CommandHistory<PushCmd> = CommandHistory::new();
        assert!(!history.undo(&mut target).unwrap());
    }

    #[test]
    fn history_redo_empty() {
        let mut target: Vec<i32> = vec![];
        let mut history: CommandHistory<PushCmd> = CommandHistory::new();
        assert!(!history.redo(&mut target).unwrap());
    }

    #[test]
    fn history_execute_clears_redo() {
        let mut target = vec![];
        let mut history = CommandHistory::new();

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.execute(PushCmd { value: 2 }, &mut target).unwrap();
        history.undo(&mut target).unwrap();
        assert!(history.can_redo());

        // New command clears redo
        history.execute(PushCmd { value: 3 }, &mut target).unwrap();
        assert!(!history.can_redo());
        assert_eq!(target, vec![1, 3]);
    }

    #[test]
    fn history_max_depth_eviction() {
        let mut target = vec![];
        let mut history = CommandHistory::with_max_depth(3);

        for i in 0..5 {
            history.execute(PushCmd { value: i }, &mut target).unwrap();
        }
        assert_eq!(history.undo_count(), 3); // oldest 2 evicted
        assert_eq!(history.max_depth(), 3);
    }

    #[test]
    fn history_clear() {
        let mut target = vec![];
        let mut history = CommandHistory::new();

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.execute(PushCmd { value: 2 }, &mut target).unwrap();
        history.undo(&mut target).unwrap();

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn history_default() {
        let history: CommandHistory<PushCmd> = CommandHistory::default();
        assert_eq!(history.max_depth(), DEFAULT_MAX_DEPTH);
        assert!(history.undo_stack.is_empty());
    }

    // -- Fallible command for error-path tests --

    #[derive(Debug)]
    struct FailReverse {
        applied: bool,
    }

    impl Command for FailReverse {
        type Target = Vec<i32>;
        type Error = String;

        fn apply(&mut self, target: &mut Vec<i32>) -> Result<(), String> {
            target.push(99);
            self.applied = true;
            Ok(())
        }

        fn reverse(&mut self, _target: &mut Vec<i32>) -> Result<(), String> {
            Err("reverse failed".into())
        }

        fn description(&self) -> &str {
            "fail reverse"
        }
    }

    #[test]
    fn history_failed_undo_preserves_command() {
        let mut target = vec![];
        let mut history: CommandHistory<FailReverse> = CommandHistory::new();
        history
            .execute(FailReverse { applied: false }, &mut target)
            .unwrap();
        assert_eq!(history.undo_count(), 1);

        let result = history.undo(&mut target);
        assert!(result.is_err());
        // Command restored to undo stack, not lost
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn history_failed_redo_preserves_command() {
        #[derive(Debug)]
        struct FailSecondApply {
            count: usize,
        }

        impl Command for FailSecondApply {
            type Target = Vec<i32>;
            type Error = String;
            fn apply(&mut self, target: &mut Vec<i32>) -> Result<(), String> {
                self.count += 1;
                if self.count > 1 {
                    return Err("second apply failed".into());
                }
                target.push(1);
                Ok(())
            }
            fn reverse(&mut self, target: &mut Vec<i32>) -> Result<(), String> {
                target.pop();
                Ok(())
            }
            fn description(&self) -> &str {
                "fail second apply"
            }
        }

        let mut target = vec![];
        let mut history = CommandHistory::new();
        history
            .execute(FailSecondApply { count: 0 }, &mut target)
            .unwrap();
        history.undo(&mut target).unwrap();
        assert_eq!(history.redo_count(), 1);

        let result = history.redo(&mut target);
        assert!(result.is_err());
        // Command restored to redo stack, not lost
        assert_eq!(history.redo_count(), 1);
        assert_eq!(history.undo_count(), 0);
    }

    #[test]
    fn history_max_depth_one() {
        let mut target = vec![];
        let mut history = CommandHistory::with_max_depth(1);

        history.execute(PushCmd { value: 1 }, &mut target).unwrap();
        history.execute(PushCmd { value: 2 }, &mut target).unwrap();
        assert_eq!(history.undo_count(), 1);

        // Can only undo the last command
        history.undo(&mut target).unwrap();
        assert_eq!(target, vec![1]); // value 2 undone, value 1 remains (its command was evicted)
    }

    #[test]
    fn history_compound_through_history() {
        let mut target = vec![];
        let mut history: CommandHistory<CompoundCommand<PushCmd>> = CommandHistory::new();

        let compound = CompoundCommand::new(
            "batch push",
            vec![
                PushCmd { value: 10 },
                PushCmd { value: 20 },
                PushCmd { value: 30 },
            ],
        );
        history.execute(compound, &mut target).unwrap();
        assert_eq!(target, vec![10, 20, 30]);
        assert_eq!(history.undo_count(), 1);

        history.undo(&mut target).unwrap();
        assert!(target.is_empty());

        history.redo(&mut target).unwrap();
        assert_eq!(target, vec![10, 20, 30]);
    }
}
