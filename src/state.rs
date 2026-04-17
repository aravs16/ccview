use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// User interaction state for a single page.
/// Stored separately from page content so CC can overwrite pages without losing user state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageState {
    /// Checklist toggles: block_id -> { item_index -> checked }
    #[serde(default)]
    pub checks: HashMap<String, HashMap<usize, bool>>,

    /// Collapsed sections: block_id -> collapsed
    #[serde(default)]
    pub collapsed: HashMap<String, bool>,

    /// Table sort: block_id -> { column, desc }
    #[serde(default)]
    pub table_sort: HashMap<String, TableSort>,

    /// Scroll position
    #[serde(default)]
    pub scroll: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSort {
    pub column: String,
    pub desc: bool,
}

fn state_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".ccview").join("state")
}

fn state_path(page_id: &str, channel: &str) -> PathBuf {
    let dir = state_dir().join(channel);
    fs::create_dir_all(&dir).ok();
    dir.join(format!("{}.state.json", page_id))
}

impl PageState {
    pub fn load(page_id: &str, channel: &str) -> Self {
        let path = state_path(page_id, channel);
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, page_id: &str, channel: &str) {
        let path = state_path(page_id, channel);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            fs::write(&path, json).ok();
        }
    }

    pub fn is_checked(&self, block_id: &str, item_index: usize) -> Option<bool> {
        self.checks
            .get(block_id)
            .and_then(|m| m.get(&item_index))
            .copied()
    }

    pub fn toggle_check(&mut self, block_id: &str, item_index: usize) {
        let block_checks = self.checks.entry(block_id.to_string()).or_default();
        let current = block_checks.get(&item_index).copied().unwrap_or(false);
        block_checks.insert(item_index, !current);
    }

    pub fn is_collapsed(&self, block_id: &str) -> Option<bool> {
        self.collapsed.get(block_id).copied()
    }

    pub fn toggle_collapsed(&mut self, block_id: &str) {
        let current = self.collapsed.get(block_id).copied().unwrap_or(false);
        self.collapsed.insert(block_id.to_string(), !current);
    }

    pub fn get_sort(&self, block_id: &str) -> Option<&TableSort> {
        self.table_sort.get(block_id)
    }

    pub fn cycle_sort(&mut self, block_id: &str, column: &str) {
        let current = self.table_sort.get(block_id);
        let new_sort = match current {
            Some(s) if s.column == column && !s.desc => TableSort {
                column: column.to_string(),
                desc: true,
            },
            Some(s) if s.column == column && s.desc => {
                self.table_sort.remove(block_id);
                return;
            }
            _ => TableSort {
                column: column.to_string(),
                desc: false,
            },
        };
        self.table_sort.insert(block_id.to_string(), new_sort);
    }
}
