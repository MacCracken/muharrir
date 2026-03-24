//! Integration tests for muharrir.

#[cfg(feature = "hierarchy")]
use muharrir::hierarchy::{build_hierarchy, flatten};

#[cfg(feature = "inspector")]
use muharrir::inspector::{Property, PropertySheet};

#[cfg(feature = "history")]
use muharrir::history::{Action, History};

#[cfg(feature = "expr")]
use muharrir::expr::{eval_f64, eval_or};

#[cfg(all(feature = "inspector", feature = "expr"))]
#[test]
fn expr_driven_inspector_field() {
    let value = eval_f64("2 * pi").unwrap();
    let mut sheet = PropertySheet::new();
    sheet.push(Property::new("Transform", "x", format!("{value:.2}")));
    assert!(sheet.properties[0].value.contains("6.28"));
}

#[cfg(all(feature = "history", feature = "inspector"))]
#[test]
fn history_tracks_property_changes() {
    let mut history = History::new();
    let mut sheet = PropertySheet::new();

    // Initial state
    sheet.push(Property::new("Transform", "x", "0.0"));

    // Record a change
    history.record(
        "inspector",
        Action::new(
            "set_property",
            serde_json::json!({"category": "Transform", "name": "x", "before": "0.0", "after": "5.0"}),
        ),
    );

    assert_eq!(history.len(), 1);
    assert!(history.verify());

    // Undo
    let entry = history.undo().unwrap();
    assert_eq!(entry.action(), "set_property");
}

#[cfg(feature = "hierarchy")]
#[test]
fn hierarchy_with_inspector() {
    let parents = std::collections::HashMap::from([(2u64, 1u64), (3, 1)]);
    let tree = build_hierarchy(
        &[1, 2, 3],
        |id| parents.get(&id).copied(),
        |id| format!("Node {id}"),
    );
    let flat = flatten(&tree);
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].depth, 0);
}

#[cfg(feature = "hw")]
#[test]
fn hardware_detection_works() {
    let profile = muharrir::HardwareProfile::detect();
    assert!(!profile.device_name.is_empty());
}

#[cfg(all(feature = "command", feature = "inspector"))]
#[test]
fn command_modifies_inspector() {
    use muharrir::command::{Command, CommandHistory};
    use std::convert::Infallible;

    #[derive(Debug)]
    struct SetValue {
        name: &'static str,
        old_value: String,
        new_value: String,
    }

    impl Command for SetValue {
        type Target = PropertySheet;
        type Error = Infallible;

        fn apply(&mut self, target: &mut PropertySheet) -> Result<(), Infallible> {
            if let Some(p) = target.properties.iter_mut().find(|p| p.name == self.name) {
                self.old_value = std::mem::replace(&mut p.value, self.new_value.clone());
            }
            Ok(())
        }

        fn reverse(&mut self, target: &mut PropertySheet) -> Result<(), Infallible> {
            if let Some(p) = target.properties.iter_mut().find(|p| p.name == self.name) {
                p.value = self.old_value.clone();
            }
            Ok(())
        }

        fn description(&self) -> &str {
            "set value"
        }
    }

    let mut sheet = PropertySheet::new();
    sheet.push(Property::new("Transform", "x", "0.0"));

    let mut history = CommandHistory::new();
    history
        .execute(
            SetValue {
                name: "x",
                old_value: String::new(),
                new_value: "5.0".into(),
            },
            &mut sheet,
        )
        .unwrap();

    assert_eq!(sheet.properties[0].value, "5.0");

    history.undo(&mut sheet).unwrap();
    assert_eq!(sheet.properties[0].value, "0.0");

    history.redo(&mut sheet).unwrap();
    assert_eq!(sheet.properties[0].value, "5.0");
}

#[cfg(all(feature = "command", feature = "history"))]
#[test]
fn command_with_audit_trail() {
    use muharrir::command::{Command, CommandHistory};
    use std::convert::Infallible;

    #[derive(Debug)]
    struct Inc;

    impl Command for Inc {
        type Target = i32;
        type Error = Infallible;
        fn apply(&mut self, t: &mut i32) -> Result<(), Infallible> {
            *t += 1;
            Ok(())
        }
        fn reverse(&mut self, t: &mut i32) -> Result<(), Infallible> {
            *t -= 1;
            Ok(())
        }
        fn description(&self) -> &str {
            "increment"
        }
    }

    let mut value = 0i32;
    let mut commands = CommandHistory::new();
    let mut audit = History::new();

    // Execute command and record in audit trail
    commands.execute(Inc, &mut value).unwrap();
    audit.record(
        "command",
        Action::new("increment", serde_json::json!({"value": value})),
    );

    commands.execute(Inc, &mut value).unwrap();
    audit.record(
        "command",
        Action::new("increment", serde_json::json!({"value": value})),
    );

    assert_eq!(value, 2);
    assert_eq!(commands.undo_count(), 2);
    assert_eq!(audit.len(), 2);
    assert!(audit.verify());
}

#[cfg(all(feature = "history", feature = "expr"))]
#[test]
fn full_editor_workflow() {
    let mut history = History::new();

    // User types expression into a property field
    let x = eval_f64("sin(pi/4)").unwrap();
    let y = eval_or("bad_input", 0.0);

    // Record the action
    history.record(
        "inspector",
        Action::new("set_transform", serde_json::json!({"x": x, "y": y})),
    );

    // Record another
    history.record(
        "hierarchy",
        Action::new("reparent", serde_json::json!({"child": 5, "parent": 1})),
    );

    assert_eq!(history.len(), 2);
    assert!(history.verify());

    // Undo both
    history.undo();
    history.undo();
    assert_eq!(history.cursor(), 0);

    // Redo one
    history.redo();
    assert_eq!(history.cursor(), 1);
}
