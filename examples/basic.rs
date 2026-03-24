//! Basic usage of muharrir editor primitives.

fn main() {
    // -- Hierarchy --
    let parents = std::collections::HashMap::from([(2u64, 1u64), (3, 1), (4, 2)]);
    let tree = muharrir::build_hierarchy(
        &[1, 2, 3, 4],
        |id| parents.get(&id).copied(),
        |id| match id {
            1 => "Root".into(),
            2 => "Child A".into(),
            3 => "Child B".into(),
            4 => "Grandchild".into(),
            _ => format!("Node {id}"),
        },
    );

    println!("Hierarchy:");
    for entry in muharrir::flatten(&tree) {
        println!("  {}{}", "  ".repeat(entry.depth), entry.name);
    }

    // -- Inspector --
    let mut sheet = muharrir::PropertySheet::new();
    sheet.push(muharrir::Property::new(
        "Transform",
        "position",
        "(1, 2, 3)",
    ));
    sheet.push(muharrir::Property::new(
        "Transform",
        "rotation",
        "(0, 0, 0)",
    ));
    sheet.push(muharrir::Property::new("Material", "color", "red"));

    println!("\nInspector:");
    for cat in sheet.categories() {
        println!("  [{cat}]");
        for prop in sheet.by_category(cat) {
            println!("    {}: {}", prop.name, prop.value);
        }
    }

    // -- Expression Eval --
    #[cfg(feature = "expr")]
    {
        let val = muharrir::eval_f64("2 * pi + sqrt(9)").unwrap();
        println!("\nExpression: 2 * pi + sqrt(9) = {val:.4}");
    }

    // -- Hardware --
    #[cfg(feature = "hw")]
    {
        let hw = muharrir::HardwareProfile::detect();
        println!("\nHardware: {} ({})", hw.device_name, hw.quality);
        println!("  GPU memory: {}", hw.gpu_memory_display());
    }

    println!("\nMuharrir — shared editor primitives for AGNOS");
}
