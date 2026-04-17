use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_ttl")]
    pub ttl: String,
    pub blocks: Vec<ContentBlock>,
}

fn default_ttl() -> String {
    "7d".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlock {
    Metrics {
        #[serde(default)]
        id: Option<String>,
        items: Vec<MetricItem>,
    },
    Table {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        title: Option<String>,
        columns: Vec<ColumnDef>,
        rows: Vec<serde_json::Value>,
        #[serde(default)]
        sortable: bool,
    },
    Chart {
        #[serde(default)]
        title: Option<String>,
        chart: String,
        #[serde(default)]
        data: Vec<DataPoint>,
    },
    Markdown { content: String },
    Callout {
        style: String,
        #[serde(default)]
        title: Option<String>,
        content: String,
    },
    Timeline {
        #[serde(default)]
        title: Option<String>,
        items: Vec<TimelineItem>,
    },
    Kv {
        #[serde(default)]
        title: Option<String>,
        items: Vec<KvItem>,
    },
    Code {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        language: Option<String>,
        content: String,
    },
    Progress { items: Vec<ProgressItem> },
    List {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default = "default_list_style")]
        style: String,
        items: Vec<CListItem>,
    },
    Divider {
        #[serde(default)]
        label: Option<String>,
    },
    Section {
        #[serde(default)]
        id: Option<String>,
        title: String,
        #[serde(default)]
        collapsed: bool,
        blocks: Vec<ContentBlock>,
    },
}

fn default_list_style() -> String {
    "bullet".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricItem {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub change: Option<String>,
    #[serde(default)]
    pub sentiment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColumnDef {
    Simple(String),
    Full { key: String, label: String, #[serde(default)] align: Option<String> },
}

impl ColumnDef {
    pub fn key(&self) -> &str {
        match self {
            ColumnDef::Simple(s) => s,
            ColumnDef::Full { key, .. } => key,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            ColumnDef::Simple(s) => s,
            ColumnDef::Full { label, .. } => label,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItem {
    pub date: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvItem {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressItem {
    pub label: String,
    pub value: f64,
    pub max: f64,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CListItem {
    Simple(String),
    Check { text: String, #[serde(default)] checked: bool },
}

impl CListItem {
    pub fn text(&self) -> &str {
        match self {
            CListItem::Simple(s) => s,
            CListItem::Check { text, .. } => text,
        }
    }
    pub fn is_checked(&self) -> Option<bool> {
        match self {
            CListItem::Simple(_) => None,
            CListItem::Check { checked, .. } => Some(*checked),
        }
    }
}

impl ContentBlock {
    pub fn block_id(&self, index: usize) -> String {
        let explicit = match self {
            ContentBlock::Metrics { id, .. } => id.as_deref(),
            ContentBlock::Table { id, .. } => id.as_deref(),
            ContentBlock::List { id, .. } => id.as_deref(),
            ContentBlock::Section { id, .. } => id.as_deref(),
            _ => None,
        };
        explicit.map(|s| s.to_string()).unwrap_or_else(|| format!("block-{}", index))
    }
}

/// Loaded page with its file id and channel
#[derive(Debug, Clone)]
pub struct LoadedPage {
    pub id: String,
    pub channel: String,
    pub page: Page,
}
