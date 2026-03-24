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
// Command benchmarks
// ---------------------------------------------------------------------------

#[cfg(feature = "command")]
mod command_bench {
    use super::*;
    use muharrir::command::{Command, CommandHistory, CompoundCommand};
    use std::convert::Infallible;

    #[derive(Debug)]
    struct PushCmd(i32);

    impl Command for PushCmd {
        type Target = Vec<i32>;
        type Error = Infallible;
        fn apply(&mut self, t: &mut Vec<i32>) -> Result<(), Infallible> {
            t.push(self.0);
            Ok(())
        }
        fn reverse(&mut self, t: &mut Vec<i32>) -> Result<(), Infallible> {
            t.pop();
            Ok(())
        }
        fn description(&self) -> &str {
            "push"
        }
    }

    pub fn bench_command_execute_100(c: &mut Criterion) {
        c.bench_function("command_execute_100", |b| {
            b.iter(|| {
                let mut target = Vec::with_capacity(100);
                let mut history = CommandHistory::with_max_depth(200);
                for i in 0..100 {
                    history.execute(PushCmd(i), &mut target).unwrap();
                }
                black_box(&target);
            });
        });
    }

    pub fn bench_command_undo_redo_cycle(c: &mut Criterion) {
        let mut target = Vec::new();
        let mut history = CommandHistory::with_max_depth(200);
        for i in 0..100 {
            history.execute(PushCmd(i), &mut target).unwrap();
        }
        c.bench_function("command_undo_redo_cycle", |b| {
            b.iter(|| {
                history.undo(&mut target).unwrap();
                history.redo(&mut target).unwrap();
                black_box(&target);
            });
        });
    }

    pub fn bench_command_compound_10(c: &mut Criterion) {
        c.bench_function("command_compound_10", |b| {
            b.iter(|| {
                let mut target = Vec::new();
                let cmds: Vec<PushCmd> = (0..10).map(PushCmd).collect();
                let mut compound = CompoundCommand::new("batch", cmds);
                compound.apply(&mut target).unwrap();
                black_box(&target);
            });
        });
    }
}

#[cfg(feature = "command")]
criterion_group!(
    command_benches,
    command_bench::bench_command_execute_100,
    command_bench::bench_command_undo_redo_cycle,
    command_bench::bench_command_compound_10
);

// ---------------------------------------------------------------------------
// Notification benchmarks
// ---------------------------------------------------------------------------

#[cfg(feature = "notification")]
mod notification_bench {
    use super::*;
    use muharrir::notification::{NotificationLog, Severity, Toasts};

    pub fn bench_toast_push_gc(c: &mut Criterion) {
        c.bench_function("toast_push_gc_100", |b| {
            b.iter(|| {
                let mut toasts = Toasts::new();
                for _ in 0..100 {
                    toasts.push("benchmark msg", Severity::Info);
                }
                toasts.gc();
                black_box(toasts.len());
            });
        });
    }

    pub fn bench_notification_log_push(c: &mut Criterion) {
        c.bench_function("notification_log_push_100", |b| {
            b.iter(|| {
                let mut log = NotificationLog::with_max_entries(200);
                for i in 0..100 {
                    log.push(black_box("notification message"), Severity::Info, "bench");
                    black_box(i);
                }
                black_box(log.len());
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Selection benchmarks
// ---------------------------------------------------------------------------

#[cfg(feature = "selection")]
mod selection_bench {
    use super::*;
    use muharrir::selection::Selection;

    pub fn bench_selection_toggle_100(c: &mut Criterion) {
        c.bench_function("selection_toggle_100", |b| {
            b.iter(|| {
                let mut sel = Selection::new();
                for i in 0u64..100 {
                    sel.toggle(i);
                }
                black_box(sel.len());
            });
        });
    }

    pub fn bench_selection_contains(c: &mut Criterion) {
        let mut sel = Selection::new();
        for i in 0u64..100 {
            sel.add(i);
        }
        c.bench_function("selection_contains_in_100", |b| {
            b.iter(|| {
                black_box(sel.contains(&50));
                black_box(sel.contains(&999));
            });
        });
    }
}

#[cfg(feature = "selection")]
criterion_group!(
    selection_benches,
    selection_bench::bench_selection_toggle_100,
    selection_bench::bench_selection_contains
);

#[cfg(feature = "notification")]
criterion_group!(
    notification_benches,
    notification_bench::bench_toast_push_gc,
    notification_bench::bench_notification_log_push
);

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[cfg(all(
    feature = "expr",
    feature = "history",
    feature = "hw",
    feature = "command",
    feature = "notification",
    feature = "selection"
))]
criterion_main!(
    hierarchy_benches,
    inspector_benches,
    expr_benches,
    history_benches,
    hw_benches,
    command_benches,
    notification_benches,
    selection_benches
);

#[cfg(not(all(
    feature = "expr",
    feature = "history",
    feature = "hw",
    feature = "command",
    feature = "notification",
    feature = "selection"
)))]
criterion_main!(hierarchy_benches, inspector_benches);
