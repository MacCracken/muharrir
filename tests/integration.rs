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
        Action::new(
            "set_transform",
            serde_json::json!({"x": x, "y": y}),
        ),
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
