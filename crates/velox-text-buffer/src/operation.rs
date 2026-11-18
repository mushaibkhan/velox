use velox_core::Selection;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Insert {
        char_offset: usize,
        text: String,
    },
    Delete {
        char_offset: usize,
        text: String,
    },
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub timestamp: Instant,
    pub operations: Vec<Operation>,
    pub cursors_before: Vec<Selection>,
    pub cursors_after: Vec<Selection>,
}

impl Transaction {
    pub fn new(cursors_before: Vec<Selection>) -> Self {
        Self {
            timestamp: Instant::now(),
            operations: Vec::new(),
            cursors_before,
            cursors_after: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn invert(&self) -> Self {
        let mut inverted_ops = Vec::with_capacity(self.operations.len());
        for op in self.operations.iter().rev() {
            match op {
                Operation::Insert { char_offset, text } => {
                    inverted_ops.push(Operation::Delete {
                        char_offset: *char_offset,
                        text: text.clone(),
                    });
                }
                Operation::Delete { char_offset, text } => {
                    inverted_ops.push(Operation::Insert {
                        char_offset: *char_offset,
                        text: text.clone(),
                    });
                }
            }
        }

        Self {
            timestamp: Instant::now(),
            operations: inverted_ops,
            cursors_before: self.cursors_after.clone(),
            cursors_after: self.cursors_before.clone(),
        }
    }
}
