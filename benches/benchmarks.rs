use criterion::{Criterion, black_box, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Hierarchy benchmarks
// ---------------------------------------------------------------------------

fn bench_hierarchy_flat_100(c: &mut Criterion) {
    c.bench_function("hierarchy_flat_100", |b| {
        let ids: Vec<u64> = (0..100).collect();
        b.iter(|| {
            black_box(muharrir::build_hierarchy(
                &ids,
                |_| None,
                |id| format!("N{id}"),
            ))
        });
    });
}

fn bench_hierarchy_deep_20(c: &mut Criterion) {
    let ids: Vec<u64> = (0..20).collect();
    c.bench_function("hierarchy_deep_20", |b| {
        b.iter(|| {
            black_box(muharrir::build_hierarchy(
                &ids,
                |id| if id > 0 { Some(id - 1) } else { None },
                |id| format!("L{id}"),
            ))
        });
    });
}

fn bench_flatten_50(c: &mut Criterion) {
    let ids: Vec<u64> = (0..50).collect();
    let tree = muharrir::build_hierarchy(
        &ids,
        |id| if id > 0 { Some(0) } else { None },
        |id| format!("N{id}"),
    );
    c.bench_function("flatten_50_nodes", |b| {
        b.iter(|| black_box(muharrir::flatten(&tree)));
    });
}

criterion_group!(
    hierarchy_benches,
    bench_hierarchy_flat_100,
    bench_hierarchy_deep_20,
    bench_flatten_50
);

// ---------------------------------------------------------------------------
// Inspector benchmarks
// ---------------------------------------------------------------------------

fn bench_property_sheet_build(c: &mut Criterion) {
    c.bench_function("property_sheet_build_20", |b| {
        b.iter(|| {
            let mut sheet = muharrir::PropertySheet::new();
            for i in 0..20 {
                sheet.push(muharrir::Property::new(
                    "Transform",
                    "field",
                    format!("{i}"),
                ));
            }
            black_box(sheet);
        });
    });
}

criterion_group!(inspector_benches, bench_property_sheet_build);

// ---------------------------------------------------------------------------
// Expression benchmarks
// ---------------------------------------------------------------------------

#[cfg(feature = "expr")]
fn bench_eval_arithmetic(c: &mut Criterion) {
    c.bench_function("eval_arithmetic", |b| {
        b.iter(|| black_box(muharrir::eval_f64("1 + 2 * 3")));
    });
}

#[cfg(feature = "expr")]
fn bench_eval_complex(c: &mut Criterion) {
    c.bench_function("eval_complex", |b| {
        b.iter(|| black_box(muharrir::eval_f64("sqrt(sin(pi/4)^2 + cos(pi/4)^2)")));
    });
}

#[cfg(feature = "expr")]
criterion_group!(expr_benches, bench_eval_arithmetic, bench_eval_complex);

// ---------------------------------------------------------------------------
// History benchmarks
// ---------------------------------------------------------------------------

#[cfg(feature = "history")]
fn bench_history_record(c: &mut Criterion) {
    c.bench_function("history_record", |b| {
        let mut h = muharrir::History::new();
        let details = serde_json::json!({"before": 0, "after": 1});
        b.iter(|| {
            h.record("bench", muharrir::Action::new("set", details.clone()));
            black_box(&h);
        });
    });
}

#[cfg(feature = "history")]
fn bench_history_undo_redo(c: &mut Criterion) {
    let mut h = muharrir::History::new();
    for i in 0..100 {
        h.record(
            "bench",
            muharrir::Action::new("a", serde_json::json!({"i": i})),
        );
    }
    c.bench_function("history_undo_redo_cycle", |b| {
        b.iter(|| {
            h.undo();
            h.redo();
            black_box(&h);
        });
    });
}

#[cfg(feature = "history")]
criterion_group!(
    history_benches,
    bench_history_record,
    bench_history_undo_redo
);

// ---------------------------------------------------------------------------
// Hardware benchmarks
// ---------------------------------------------------------------------------

#[cfg(feature = "hw")]
fn bench_hw_detect(c: &mut Criterion) {
    c.bench_function("hw_detect", |b| {
        b.iter(|| black_box(muharrir::HardwareProfile::detect()));
    });
}

#[cfg(feature = "hw")]
criterion_group!(hw_benches, bench_hw_detect);

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[cfg(all(feature = "expr", feature = "history", feature = "hw"))]
criterion_main!(
    hierarchy_benches,
    inspector_benches,
    expr_benches,
    history_benches,
    hw_benches
);

#[cfg(not(all(feature = "expr", feature = "history", feature = "hw")))]
criterion_main!(hierarchy_benches, inspector_benches);
