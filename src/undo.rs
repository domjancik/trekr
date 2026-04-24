use crate::mapping::MappingEntry;
use crate::pages::{AppPageState, MappingField};
use crate::project::Project;
use crate::timeline_fx::{TimelineContext, TimelineFxField};
use crate::ui::TimelineFlow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndoDomain {
    Timeline,
    Mappings,
    Ui,
}

impl UndoDomain {
    pub const ALL: [Self; 3] = [Self::Timeline, Self::Mappings, Self::Ui];

    pub fn label(self) -> &'static str {
        match self {
            Self::Timeline => "Timeline",
            Self::Mappings => "Mappings",
            Self::Ui => "UI",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UndoOverlayState {
    #[default]
    None,
    MappingsQuickView,
    Discoverability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineUndoState {
    pub project: Project,
    pub selected_timeline_context: TimelineContext,
    pub selected_timeline_fx_field: TimelineFxField,
    pub selected_input_fx_row: usize,
    pub selected_output_fx_row: usize,
    pub preferred_default_input_name: Option<String>,
    pub preferred_default_output_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingsUndoState {
    pub mappings: Vec<MappingEntry>,
    pub selected_mapping_index: usize,
    pub selected_mapping_field: MappingField,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiUndoState {
    pub page_state: AppPageState,
    pub timeline_flow: TimelineFlow,
    pub overlay_state: UndoOverlayState,
    pub focused_track_view: bool,
    pub direct_mapping_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndoSnapshot {
    Timeline(TimelineUndoState),
    Mappings(MappingsUndoState),
    Ui(UiUndoState),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoEntry {
    pub domain: UndoDomain,
    pub label: String,
    pub before: UndoSnapshot,
    pub after: UndoSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoTransaction {
    pub id: u64,
    pub label: String,
    pub entries: Vec<UndoEntry>,
    pub applied: bool,
}

impl UndoTransaction {
    pub fn is_single_domain(&self, domain: UndoDomain) -> bool {
        self.entries.len() == 1 && self.entries[0].domain == domain
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UndoHistory {
    pub version: u32,
    pub next_transaction_id: u64,
    pub max_entries: usize,
    pub transactions: Vec<UndoTransaction>,
    pub global_redo: Vec<u64>,
    pub timeline_redo: Vec<u64>,
    pub mappings_redo: Vec<u64>,
    pub ui_redo: Vec<u64>,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            version: 1,
            next_transaction_id: 1,
            max_entries: 256,
            transactions: Vec::new(),
            global_redo: Vec::new(),
            timeline_redo: Vec::new(),
            mappings_redo: Vec::new(),
            ui_redo: Vec::new(),
        }
    }
}

impl UndoHistory {
    pub fn push_transaction(&mut self, label: String, entries: Vec<UndoEntry>) -> bool {
        if entries.is_empty() {
            return false;
        }

        self.global_redo.clear();
        for entry in &entries {
            self.redo_stack_mut(entry.domain).clear();
        }

        self.transactions.push(UndoTransaction {
            id: self.next_transaction_id,
            label,
            entries,
            applied: true,
        });
        self.next_transaction_id += 1;
        self.prune_oldest();
        true
    }

    pub fn undo_global(&mut self) -> Option<UndoTransaction> {
        let index = self
            .transactions
            .iter()
            .rposition(|transaction| transaction.applied)?;
        let transaction = self.transactions.get_mut(index)?;
        transaction.applied = false;
        let transaction_id = transaction.id;
        let single_domain = transaction
            .entries
            .as_slice()
            .first()
            .map(|entry| (transaction.entries.len() == 1).then_some(entry.domain))
            .flatten();
        let transaction_clone = transaction.clone();
        self.global_redo.push(transaction_id);
        if let Some(domain) = single_domain {
            self.redo_stack_mut(domain).push(transaction_id);
        }
        Some(transaction_clone)
    }

    pub fn redo_global(&mut self) -> Option<UndoTransaction> {
        while let Some(id) = self.global_redo.pop() {
            let index = self
                .transactions
                .iter()
                .position(|transaction| transaction.id == id && !transaction.applied)?;
            let transaction = self.transactions.get_mut(index)?;
            transaction.applied = true;
            let transaction_id = transaction.id;
            let single_domain = transaction
                .entries
                .as_slice()
                .first()
                .map(|entry| (transaction.entries.len() == 1).then_some(entry.domain))
                .flatten();
            let transaction_clone = transaction.clone();
            if let Some(domain) = single_domain {
                remove_last_occurrence(self.redo_stack_mut(domain), transaction_id);
            }
            return Some(transaction_clone);
        }
        None
    }

    pub fn undo_domain(&mut self, domain: UndoDomain) -> Option<UndoTransaction> {
        let index = self
            .transactions
            .iter()
            .rposition(|transaction| transaction.applied && transaction.is_single_domain(domain))?;
        let transaction = self.transactions.get_mut(index)?;
        transaction.applied = false;
        let transaction_id = transaction.id;
        let transaction_clone = transaction.clone();
        self.global_redo.push(transaction_id);
        self.redo_stack_mut(domain).push(transaction_id);
        Some(transaction_clone)
    }

    pub fn redo_domain(&mut self, domain: UndoDomain) -> Option<UndoTransaction> {
        while let Some(id) = self.redo_stack_mut(domain).pop() {
            let index = self.transactions.iter().position(|transaction| {
                transaction.id == id && !transaction.applied && transaction.is_single_domain(domain)
            })?;
            let transaction = self.transactions.get_mut(index)?;
            transaction.applied = true;
            let transaction_id = transaction.id;
            let transaction_clone = transaction.clone();
            remove_last_occurrence(&mut self.global_redo, transaction_id);
            return Some(transaction_clone);
        }
        None
    }

    fn prune_oldest(&mut self) {
        if self.transactions.len() <= self.max_entries {
            return;
        }
        let remove_count = self.transactions.len() - self.max_entries;
        let removed_ids: Vec<u64> = self
            .transactions
            .iter()
            .take(remove_count)
            .map(|transaction| transaction.id)
            .collect();
        self.transactions.drain(0..remove_count);
        self.global_redo.retain(|id| !removed_ids.contains(id));
        self.timeline_redo.retain(|id| !removed_ids.contains(id));
        self.mappings_redo.retain(|id| !removed_ids.contains(id));
        self.ui_redo.retain(|id| !removed_ids.contains(id));
    }

    fn redo_stack_mut(&mut self, domain: UndoDomain) -> &mut Vec<u64> {
        match domain {
            UndoDomain::Timeline => &mut self.timeline_redo,
            UndoDomain::Mappings => &mut self.mappings_redo,
            UndoDomain::Ui => &mut self.ui_redo,
        }
    }
}

fn remove_last_occurrence(stack: &mut Vec<u64>, id: u64) {
    if let Some(index) = stack.iter().rposition(|candidate| *candidate == id) {
        stack.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{UiUndoState, UndoDomain, UndoEntry, UndoHistory, UndoOverlayState, UndoSnapshot};
    use crate::pages::AppPageState;
    use crate::ui::TimelineFlow;

    fn ui_entry(label: &str) -> UndoEntry {
        let before = UiUndoState {
            page_state: AppPageState::default(),
            timeline_flow: TimelineFlow::DownwardColumns,
            overlay_state: UndoOverlayState::None,
            focused_track_view: false,
            direct_mapping_active: false,
        };
        let mut after = before.clone();
        after.focused_track_view = true;
        UndoEntry {
            domain: UndoDomain::Ui,
            label: label.to_string(),
            before: UndoSnapshot::Ui(before),
            after: UndoSnapshot::Ui(after),
        }
    }

    #[test]
    fn domain_undo_skips_other_domains() {
        let mut history = UndoHistory::default();
        history.push_transaction(
            "Timeline".to_string(),
            vec![UndoEntry {
                domain: UndoDomain::Timeline,
                label: "Timeline".to_string(),
                before: UndoSnapshot::Ui(UiUndoState {
                    page_state: AppPageState::default(),
                    timeline_flow: TimelineFlow::DownwardColumns,
                    overlay_state: UndoOverlayState::None,
                    focused_track_view: false,
                    direct_mapping_active: false,
                }),
                after: UndoSnapshot::Ui(UiUndoState {
                    page_state: AppPageState::default(),
                    timeline_flow: TimelineFlow::AcrossRows,
                    overlay_state: UndoOverlayState::None,
                    focused_track_view: false,
                    direct_mapping_active: false,
                }),
            }],
        );
        history.push_transaction("UI".to_string(), vec![ui_entry("UI")]);

        let undone = history
            .undo_domain(UndoDomain::Timeline)
            .expect("timeline undo");
        assert_eq!(undone.label, "Timeline");
        assert!(history.transactions[0].applied == false);
        assert!(history.transactions[1].applied);
    }

    #[test]
    fn global_redo_reapplies_last_undone_transaction() {
        let mut history = UndoHistory::default();
        history.push_transaction("UI".to_string(), vec![ui_entry("UI")]);
        let undone = history.undo_global().expect("undo");
        assert_eq!(undone.label, "UI");
        let redone = history.redo_global().expect("redo");
        assert_eq!(redone.label, "UI");
        assert!(history.transactions[0].applied);
    }
}
