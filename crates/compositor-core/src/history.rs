use std::collections::HashSet;
use uuid::Uuid;

use crate::layer::{CanvasDocument, LayerId};

/// A snapshot of document state at a point in history.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentSnapshot {
    pub document: Option<CanvasDocument>,
    pub active_layer_id: Option<LayerId>,
    pub revision: Uuid,
}

#[derive(Debug, Clone, PartialEq)]
struct HistoryEntry {
    name: String,
    before: DocumentSnapshot,
    after: DocumentSnapshot,
}

/// Linear undo/redo history manager matching Compositor's DocumentHistory architecture.
pub struct DocumentHistory {
    past: Vec<HistoryEntry>,
    future: Vec<HistoryEntry>,
    revision: Uuid,
    saved_revision: Option<Uuid>,
    pending: Option<DocumentSnapshot>,
    pending_name: String,
    depth: usize,
    pub entry_limit: usize,
    pub retained_byte_limit: usize,
}

impl Default for DocumentHistory {
    fn default() -> Self {
        Self::new(100, 256 * 1024 * 1024)
    }
}

impl DocumentHistory {
    pub fn new(entry_limit: usize, retained_byte_limit: usize) -> Self {
        let rev = Uuid::new_v4();
        Self {
            past: Vec::new(),
            future: Vec::new(),
            revision: rev,
            saved_revision: Some(rev),
            pending: None,
            pending_name: "Edit".to_string(),
            depth: 0,
            entry_limit,
            retained_byte_limit,
        }
    }

    pub fn can_undo(&self) -> bool {
        self.depth == 0 && !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        self.depth == 0 && !self.future.is_empty()
    }

    pub fn undo_name(&self) -> &str {
        self.past.last().map(|e| e.name.as_str()).unwrap_or("")
    }

    pub fn redo_name(&self) -> &str {
        self.future.last().map(|e| e.name.as_str()).unwrap_or("")
    }

    pub fn is_modified(&self) -> bool {
        Some(self.revision) != self.saved_revision
    }

    pub fn undo_count(&self) -> usize {
        self.past.len()
    }

    pub fn current_revision(&self) -> Uuid {
        self.revision
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = Some(self.revision);
    }

    pub fn mark_saved_revision(&mut self, saved: Uuid) {
        self.saved_revision = Some(saved);
    }

    pub fn reset(&mut self) {
        self.past.clear();
        self.future.clear();
        self.pending = None;
        self.depth = 0;
        self.revision = Uuid::new_v4();
        self.saved_revision = Some(self.revision);
    }

    /// Begins a history transaction. Can be nested; only the outermost call records the baseline snapshot.
    pub fn begin(
        &mut self,
        name: impl Into<String>,
        document: Option<&CanvasDocument>,
        selection: Option<LayerId>,
    ) {
        if self.depth == 0 {
            self.pending = Some(DocumentSnapshot {
                document: document.cloned(),
                active_layer_id: selection,
                revision: self.revision,
            });
            self.pending_name = name.into();
        }
        self.depth += 1;
    }

    /// Ends a history transaction. If outermost and document changed, commits entry to past stack.
    pub fn end(
        &mut self,
        document: Option<&CanvasDocument>,
        selection: Option<LayerId>,
    ) {
        if self.depth == 0 {
            return;
        }
        self.depth -= 1;
        if self.depth != 0 {
            return;
        }

        let before = match self.pending.take() {
            Some(b) => b,
            None => return,
        };

        // Selecting, navigating, and no-op edits must preserve redo history.
        if before.document.as_ref() == document {
            return;
        }

        self.revision = Uuid::new_v4();
        let after = DocumentSnapshot {
            document: document.cloned(),
            active_layer_id: selection,
            revision: self.revision,
        };

        self.past.push(HistoryEntry {
            name: self.pending_name.clone(),
            before,
            after,
        });
        self.future.clear();
        self.trim(document);
    }

    /// Reverts the most recent edit in past and moves it to future.
    pub fn undo(&mut self) -> Option<DocumentSnapshot> {
        if !self.can_undo() {
            return None;
        }
        let entry = self.past.pop()?;
        self.revision = entry.before.revision;
        let before_doc = entry.before.document.clone();
        let result = entry.before.clone();
        self.future.push(entry);
        self.trim(before_doc.as_ref());
        Some(result)
    }

    /// Reapplies the most recent undone edit in future and moves it to past.
    pub fn redo(&mut self) -> Option<DocumentSnapshot> {
        if !self.can_redo() {
            return None;
        }
        let entry = self.future.pop()?;
        self.revision = entry.after.revision;
        let after_doc = entry.after.document.clone();
        let result = entry.after.clone();
        self.past.push(entry);
        self.trim(after_doc.as_ref());
        Some(result)
    }

    /// Calculates retained bytes by counting unique layer asset references in history.
    pub fn retained_bytes(&self, current: Option<&CanvasDocument>) -> usize {
        let mut seen = HashSet::new();
        if let Some(doc) = current {
            for layer in &doc.layers {
                if let Some(ref asset) = layer.asset_filename {
                    seen.insert(asset.clone());
                }
            }
        }

        let mut bytes = 0;
        for entry in self.past.iter().chain(self.future.iter()) {
            for snapshot in [&entry.before, &entry.after] {
                if let Some(ref doc) = snapshot.document {
                    for layer in &doc.layers {
                        if let Some(ref asset) = layer.asset_filename {
                            if seen.insert(asset.clone()) {
                                // Estimated 4 bytes per pixel of layer surface
                                let area = (layer.transform.size[0] * layer.transform.size[1]) as usize;
                                bytes += area * 4;
                            }
                        }
                    }
                }
            }
        }
        bytes
    }

    fn trim(&mut self, current: Option<&CanvasDocument>) {
        while self.past.len() + self.future.len() > self.entry_limit
            || self.retained_bytes(current) > self.retained_byte_limit
        {
            if !self.past.is_empty() {
                self.past.remove(0);
            } else if !self.future.is_empty() {
                self.future.remove(0);
            } else {
                break;
            }
        }
    }
}
