use crate::operation::{Operation, Transaction};
use std::time::Duration;

const TYPING_COALESCE_THRESHOLD: Duration = Duration::from_millis(400);

pub struct History {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
    max_history: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl History {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    pub fn push(&mut self, transaction: Transaction) {
        if transaction.is_empty() {
            return;
        }

        self.redo_stack.clear();

        // Check if we can coalesce consecutive single-character inserts
        if let Some(last) = self.undo_stack.last_mut() {
            if transaction.operations.len() == 1 && last.operations.len() == 1 {
                if let (
                    Operation::Insert { char_offset: last_off, text: last_text },
                    Operation::Insert { char_offset: new_off, text: new_text },
                ) = (&mut last.operations[0], &transaction.operations[0]) {
                    let is_consecutive = *last_off + last_text.chars().count() == *new_off;
                    let is_quick = transaction.timestamp.duration_since(last.timestamp) < TYPING_COALESCE_THRESHOLD;
                    let is_simple_char = !new_text.contains('\n') && !last_text.contains('\n');

                    if is_consecutive && is_quick && is_simple_char {
                        last_text.push_str(new_text);
                        last.cursors_after = transaction.cursors_after;
                        last.timestamp = transaction.timestamp;
                        return;
                    }
                }
            }
        }

        self.undo_stack.push(transaction);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    pub fn pop_undo(&mut self) -> Option<Transaction> {
        let tx = self.undo_stack.pop()?;
        let inverted = tx.invert();
        self.redo_stack.push(tx);
        Some(inverted)
    }

    pub fn pop_redo(&mut self) -> Option<Transaction> {
        let tx = self.redo_stack.pop()?;
        let redo_tx = tx.clone();
        self.undo_stack.push(tx);
        Some(redo_tx)
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
