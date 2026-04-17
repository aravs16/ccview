use crate::protocol::{LoadedPage, Page};
use std::fs;
use std::path::{Path, PathBuf};

pub fn pages_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".ccview").join("pages")
}

pub fn ensure_dir() {
    let dir = pages_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }
    // Also ensure _inbox channel
    let inbox = dir.join("_inbox");
    if !inbox.exists() {
        fs::create_dir_all(&inbox).ok();
    }
}

pub fn load_all_pages() -> Vec<LoadedPage> {
    let dir = pages_dir();
    let mut pages = Vec::new();

    // Load pages from root (legacy flat structure) → channel = "_inbox"
    load_from_dir(&dir, "_inbox", &mut pages);

    // Load pages from channel subdirectories
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let channel = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                load_from_dir(&path, &channel, &mut pages);
            }
        }
    }

    // Sort: pinned first, then by updated desc
    pages.sort_by(|a, b| {
        b.page.pinned.cmp(&a.page.pinned).then_with(|| {
            let ua = a.page.updated.as_deref().unwrap_or("");
            let ub = b.page.updated.as_deref().unwrap_or("");
            ub.cmp(ua)
        })
    });

    pages
}

fn load_from_dir(dir: &Path, default_channel: &str, pages: &mut Vec<LoadedPage>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") && !path.to_string_lossy().contains(".state.") {
                if let Some(page) = load_page(&path) {
                    let id = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let channel = page
                        .channel
                        .clone()
                        .unwrap_or_else(|| default_channel.to_string());
                    pages.push(LoadedPage { id, channel, page });
                }
            }
        }
    }
}

fn load_page(path: &Path) -> Option<Page> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Get unique channels from loaded pages, sorted with _inbox last
pub fn get_channels(pages: &[LoadedPage]) -> Vec<String> {
    let mut channels: Vec<String> = pages
        .iter()
        .map(|p| p.channel.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // Put _inbox at the end
    if let Some(pos) = channels.iter().position(|c| c == "_inbox") {
        let inbox = channels.remove(pos);
        channels.push(inbox);
    }

    channels
}
