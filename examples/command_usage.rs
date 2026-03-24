//! Demonstrates the command pattern: Command trait, CompoundCommand, CommandHistory,
//! and composition with the libro-backed audit History.

use muharrir::command::{Command, CommandHistory, CompoundCommand};
use std::convert::Infallible;

/// A simple document: just a list of lines.
#[derive(Debug, Default)]
struct Document {
    lines: Vec<String>,
}

/// Insert a line at a given index.
#[derive(Debug)]
struct InsertLine {
    index: usize,
    text: String,
}

impl Command for InsertLine {
    type Target = Document;
    type Error = Infallible;

    fn apply(&mut self, doc: &mut Document) -> Result<(), Infallible> {
        doc.lines.insert(self.index, self.text.clone());
        Ok(())
    }

    fn reverse(&mut self, doc: &mut Document) -> Result<(), Infallible> {
        doc.lines.remove(self.index);
        Ok(())
    }

    fn description(&self) -> &str {
        "insert line"
    }
}

/// Remove a line at a given index, capturing it for undo (lazy state capture).
#[derive(Debug)]
struct RemoveLine {
    index: usize,
    removed: Option<String>,
}

impl Command for RemoveLine {
    type Target = Document;
    type Error = Infallible;

    fn apply(&mut self, doc: &mut Document) -> Result<(), Infallible> {
        self.removed = Some(doc.lines.remove(self.index));
        Ok(())
    }

    fn reverse(&mut self, doc: &mut Document) -> Result<(), Infallible> {
        if let Some(text) = self.removed.take() {
            doc.lines.insert(self.index, text);
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "remove line"
    }
}

fn main() {
    let mut doc = Document::default();
    let mut history: CommandHistory<Box<dyn Command<Target = Document, Error = Infallible>>> =
        CommandHistory::with_max_depth(100);

    // Insert three lines
    for (i, text) in ["Hello", "World", "!"].iter().enumerate() {
        history
            .execute(
                Box::new(InsertLine {
                    index: i,
                    text: text.to_string(),
                }),
                &mut doc,
            )
            .unwrap();
    }
    println!("After inserts: {:?}", doc.lines);

    // Compound command: remove line 2 and insert replacement
    let compound: Box<dyn Command<Target = Document, Error = Infallible>> =
        Box::new(CompoundCommand::new(
            "replace line",
            vec![
                Box::new(RemoveLine {
                    index: 1,
                    removed: None,
                }) as Box<dyn Command<Target = Document, Error = Infallible>>,
                Box::new(InsertLine {
                    index: 1,
                    text: "Muharrir".into(),
                }),
            ],
        ));
    history.execute(compound, &mut doc).unwrap();
    println!("After replace: {:?}", doc.lines);

    // Undo the compound
    history.undo(&mut doc).unwrap();
    println!("After undo:    {:?}", doc.lines);

    // Redo it
    history.redo(&mut doc).unwrap();
    println!("After redo:    {:?}", doc.lines);

    println!(
        "\nHistory: {} undo, {} redo",
        history.undo_count(),
        history.redo_count()
    );

    // -- Audit trail composition --
    #[cfg(feature = "history")]
    {
        use muharrir::history::{Action, History};
        let mut audit = History::new();
        audit.record(
            "example",
            Action::new("insert", serde_json::json!({"lines": 3})),
        );
        audit.record(
            "example",
            Action::new("replace", serde_json::json!({"index": 1})),
        );
        println!(
            "Audit chain: {} entries, verified={}",
            audit.len(),
            audit.verify()
        );
    }
}
