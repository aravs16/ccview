use crate::protocol::*;
use crate::state::PageState;
use ratatui::prelude::*;

/// An interactive item the user can focus on
#[derive(Debug, Clone)]
pub struct FocusItem {
    pub line_index: usize,
    pub block_id: String,
    pub kind: FocusKind,
}

#[derive(Debug, Clone)]
pub enum FocusKind {
    CheckItem(usize),         // item index within the list
    SectionToggle,
    TableSort(String),        // column key
}

pub struct RenderResult {
    pub lines: Vec<Line<'static>>,
    pub focus_items: Vec<FocusItem>,
}

pub fn render_blocks(blocks: &[ContentBlock], width: u16, state: &PageState, focused: Option<usize>) -> RenderResult {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut focus_items: Vec<FocusItem> = Vec::new();
    let w = width.saturating_sub(2) as usize;

    for (idx, block) in blocks.iter().enumerate() {
        let block_id = block.block_id(idx);
        render_block(block, &block_id, w, state, focused, &mut lines, &mut focus_items);
        lines.push(Line::raw(""));
    }
    RenderResult { lines, focus_items }
}

fn render_block(
    block: &ContentBlock, block_id: &str, w: usize, state: &PageState,
    focused: Option<usize>,
    lines: &mut Vec<Line<'static>>, focus_items: &mut Vec<FocusItem>,
) {
    match block {
        ContentBlock::Metrics { items, .. } => render_metrics(items, lines),
        ContentBlock::Table { title, columns, rows, .. } => render_table(title, columns, rows, w, lines),
        ContentBlock::Chart { title, chart, data } => render_chart(title, chart, data, w, lines),
        ContentBlock::Markdown { content } => render_markdown(content, w, lines),
        ContentBlock::Callout { style, title, content } => render_callout(style, title, content, w, lines),
        ContentBlock::Timeline { title, items } => render_timeline(title, items, lines),
        ContentBlock::Kv { title, items } => render_kv(title, items, w, lines),
        ContentBlock::Code { title, content, .. } => render_code(title, content, lines),
        ContentBlock::Progress { items } => render_progress(items, w, lines),
        ContentBlock::List { title, style, items, .. } => render_list(title, style, items, block_id, w, state, focused, lines, focus_items),
        ContentBlock::Divider { label } => render_divider(label, w, lines),
        ContentBlock::Section { title, blocks, .. } => {
            let collapsed = state.is_collapsed(block_id).unwrap_or(false);
            let arrow = if collapsed { "▸" } else { "▾" };

            let fi = focus_items.len();
            let is_focused = focused == Some(fi);
            focus_items.push(FocusItem { line_index: lines.len(), block_id: block_id.to_string(), kind: FocusKind::SectionToggle });

            let style = if is_focused {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            };
            lines.push(Line::from(Span::styled(format!("{} {}", arrow, title), style)));

            if !collapsed {
                for (si, b) in blocks.iter().enumerate() {
                    let sub_id = format!("{}.{}", block_id, si);
                    render_block(b, &sub_id, w, state, focused, lines, focus_items);
                }
            }
        }
    }
}

fn render_metrics(items: &[MetricItem], lines: &mut Vec<Line<'static>>) {
    let mut label_spans: Vec<Span<'static>> = Vec::new();
    let mut value_spans: Vec<Span<'static>> = Vec::new();
    let mut change_spans: Vec<Span<'static>> = Vec::new();

    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            value_spans.push(Span::styled("  │  ".to_string(), Style::default().fg(Color::DarkGray)));
            label_spans.push(Span::raw("     ".to_string()));
            change_spans.push(Span::raw("     ".to_string()));
        }
        let color = match item.sentiment.as_deref() {
            Some("positive") => Color::Green,
            Some("negative") => Color::Red,
            _ => Color::White,
        };
        value_spans.push(Span::styled(item.value.clone(), Style::default().fg(color).add_modifier(Modifier::BOLD)));
        label_spans.push(Span::styled(item.label.clone(), Style::default().fg(Color::DarkGray)));
        change_spans.push(Span::styled(
            item.change.clone().unwrap_or_default(),
            Style::default().fg(color),
        ));
    }
    lines.push(Line::from(label_spans));
    lines.push(Line::from(value_spans));
    lines.push(Line::from(change_spans));
}

fn render_table(title: &Option<String>, columns: &[ColumnDef], rows: &[serde_json::Value], w: usize, lines: &mut Vec<Line<'static>>) {
    if let Some(t) = title {
        lines.push(Line::from(Span::styled(t.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
        lines.push(Line::raw(""));
    }
    let col_count = columns.len();
    let col_w = if col_count > 0 { w / col_count } else { w };

    let header_spans: Vec<Span<'static>> = columns.iter()
        .map(|c| Span::styled(pad(c.label(), col_w), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)))
        .collect();
    lines.push(Line::from(header_spans));
    lines.push(Line::from(Span::styled("─".repeat(w.min(120)), Style::default().fg(Color::DarkGray))));

    for row in rows {
        let row_spans: Vec<Span<'static>> = columns.iter().map(|c| {
            let val = match row {
                serde_json::Value::Object(map) => {
                    map.get(c.key())
                        .or_else(|| map.get(&c.label().to_lowercase()))
                        .map(|v| match v { serde_json::Value::String(s) => s.clone(), o => o.to_string() })
                        .unwrap_or_default()
                }
                _ => String::new(),
            };
            Span::styled(pad(&val, col_w), Style::default().fg(Color::Gray))
        }).collect();
        lines.push(Line::from(row_spans));
    }
}

fn render_chart(title: &Option<String>, _chart_type: &str, data: &[DataPoint], w: usize, lines: &mut Vec<Line<'static>>) {
    if let Some(t) = title {
        lines.push(Line::from(Span::styled(t.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
        lines.push(Line::raw(""));
    }
    if data.is_empty() { return; }

    let max_val = data.iter().map(|d| d.value).fold(0.0f64, f64::max);
    let label_w = data.iter().map(|d| d.label.len()).max().unwrap_or(6);
    let bar_max = w.saturating_sub(label_w + 12);

    for dp in data {
        let bar_len = if max_val > 0.0 { ((dp.value / max_val) * bar_max as f64) as usize } else { 0 };
        lines.push(Line::from(vec![
            Span::styled(pad(&dp.label, label_w + 1), Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(bar_len), Style::default().fg(Color::Cyan)),
            Span::styled(format!(" {}", fmt_num(dp.value)), Style::default().fg(Color::Gray)),
        ]));
    }
}

fn render_markdown(content: &str, w: usize, lines: &mut Vec<Line<'static>>) {
    for raw in content.lines() {
        if let Some(h) = raw.strip_prefix("## ") {
            lines.push(Line::from(Span::styled(h.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
        } else if let Some(h) = raw.strip_prefix("# ") {
            lines.push(Line::from(Span::styled(h.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))));
        } else if let Some(item) = raw.strip_prefix("- ").or_else(|| raw.strip_prefix("* ")) {
            for wrapped in textwrap::wrap(item, w.saturating_sub(4)) {
                lines.push(Line::from(vec![
                    Span::styled("  • ".to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(wrapped.to_string(), Style::default().fg(Color::Gray)),
                ]));
            }
        } else if raw.is_empty() {
            lines.push(Line::raw(""));
        } else {
            for wrapped in textwrap::wrap(raw, w) {
                lines.push(Line::from(Span::styled(wrapped.to_string(), Style::default().fg(Color::Gray))));
            }
        }
    }
}

fn render_callout(style: &str, title: &Option<String>, content: &str, w: usize, lines: &mut Vec<Line<'static>>) {
    let (icon, color) = match style {
        "success" => ("✓", Color::Green), "warning" => ("!", Color::Yellow),
        "error" => ("✗", Color::Red), _ => ("i", Color::Cyan),
    };
    if let Some(t) = title {
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(t.clone(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]));
    }
    for wrapped in textwrap::wrap(content, w.saturating_sub(4)) {
        lines.push(Line::from(vec![
            Span::raw("   ".to_string()),
            Span::styled(wrapped.to_string(), Style::default().fg(Color::Gray)),
        ]));
    }
}

fn render_timeline(title: &Option<String>, items: &[TimelineItem], lines: &mut Vec<Line<'static>>) {
    if let Some(t) = title {
        lines.push(Line::from(Span::styled(t.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
        lines.push(Line::raw(""));
    }
    let date_w = items.iter().map(|i| i.date.len()).max().unwrap_or(8);

    for (i, item) in items.iter().enumerate() {
        let dot_color = match item.color.as_deref() {
            Some("green") => Color::Green, Some("red") => Color::Red,
            Some("blue") => Color::Cyan, Some("yellow") => Color::Yellow, _ => Color::Gray,
        };
        let connector = if i < items.len() - 1 { "│" } else { " " };

        lines.push(Line::from(vec![
            Span::styled(pad(&item.date, date_w + 1), Style::default().fg(Color::DarkGray)),
            Span::styled("● ".to_string(), Style::default().fg(dot_color)),
            Span::styled(item.title.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        if let Some(desc) = &item.description {
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(date_w + 1)),
                Span::styled(format!("{} ", connector), Style::default().fg(Color::DarkGray)),
                Span::styled(desc.clone(), Style::default().fg(Color::Gray)),
            ]));
        }
    }
}

fn render_kv(title: &Option<String>, items: &[KvItem], w: usize, lines: &mut Vec<Line<'static>>) {
    if let Some(t) = title {
        lines.push(Line::from(Span::styled(t.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
        lines.push(Line::raw(""));
    }
    let key_w = items.iter().map(|i| i.key.len()).max().unwrap_or(10).min(w / 3);
    for item in items {
        lines.push(Line::from(vec![
            Span::styled(pad(&item.key, key_w + 2), Style::default().fg(Color::DarkGray)),
            Span::styled(item.value.clone(), Style::default().fg(Color::White)),
        ]));
    }
}

fn render_code(title: &Option<String>, content: &str, lines: &mut Vec<Line<'static>>) {
    if let Some(t) = title {
        lines.push(Line::from(Span::styled(t.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
    }
    lines.push(Line::from(Span::styled("───".to_string(), Style::default().fg(Color::DarkGray))));
    for cl in content.lines() {
        lines.push(Line::from(Span::styled(format!("  {}", cl), Style::default().fg(Color::Cyan))));
    }
    lines.push(Line::from(Span::styled("───".to_string(), Style::default().fg(Color::DarkGray))));
}

fn render_progress(items: &[ProgressItem], w: usize, lines: &mut Vec<Line<'static>>) {
    let label_w = items.iter().map(|i| i.label.len()).max().unwrap_or(10);
    let bar_w = w.saturating_sub(label_w + 10);

    for item in items {
        let pct = if item.max > 0.0 { item.value / item.max } else { 0.0 };
        let filled = (pct * bar_w as f64) as usize;
        let empty = bar_w.saturating_sub(filled);
        let color = if pct > 1.0 { Color::Red } else {
            match item.color.as_deref() {
                Some("red") => Color::Red, Some("green") => Color::Green, _ => Color::Cyan,
            }
        };
        lines.push(Line::from(vec![
            Span::styled(pad(&item.label, label_w + 1), Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {:.0}%", pct * 100.0), Style::default().fg(Color::Gray)),
        ]));
    }
}

fn render_list(
    title: &Option<String>, style: &str, items: &[CListItem],
    block_id: &str, w: usize, state: &PageState, focused: Option<usize>,
    lines: &mut Vec<Line<'static>>, focus_items: &mut Vec<FocusItem>,
) {
    if let Some(t) = title {
        lines.push(Line::from(Span::styled(t.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
        lines.push(Line::raw(""));
    }
    let is_checklist = style == "checklist";

    for (item_idx, item) in items.iter().enumerate() {
        // State overrides JSON for check status
        let checked = if is_checklist {
            state.is_checked(block_id, item_idx)
                .or_else(|| item.is_checked())
                .unwrap_or(false)
        } else {
            false
        };

        let fi = focus_items.len();
        let is_focused = focused == Some(fi);

        if is_checklist {
            focus_items.push(FocusItem {
                line_index: lines.len(),
                block_id: block_id.to_string(),
                kind: FocusKind::CheckItem(item_idx),
            });
        }

        let prefix = if is_checklist {
            if checked {
                Span::styled(" ✓ ".to_string(), Style::default().fg(Color::Green))
            } else if is_focused {
                Span::styled(" ○ ".to_string(), Style::default().fg(Color::Cyan))
            } else {
                Span::styled(" ○ ".to_string(), Style::default().fg(Color::DarkGray))
            }
        } else {
            Span::styled(" • ".to_string(), Style::default().fg(Color::DarkGray))
        };

        let text_color = if checked {
            Color::DarkGray
        } else if is_focused {
            Color::White
        } else {
            Color::Gray
        };

        let focus_marker = if is_focused { "› " } else { "  " };

        for (i, wrapped) in textwrap::wrap(item.text(), w.saturating_sub(6)).iter().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(focus_marker.to_string(), Style::default().fg(Color::Cyan)),
                    prefix.clone(),
                    Span::styled(
                        wrapped.to_string(),
                        Style::default().fg(text_color).add_modifier(if checked { Modifier::CROSSED_OUT } else { Modifier::empty() }),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("     ".to_string()),
                    Span::styled(wrapped.to_string(), Style::default().fg(text_color)),
                ]));
            }
        }
    }
}

fn render_divider(label: &Option<String>, w: usize, lines: &mut Vec<Line<'static>>) {
    if let Some(l) = label {
        let dw = w.saturating_sub(l.len() + 2) / 2;
        lines.push(Line::from(vec![
            Span::styled("─".repeat(dw), Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {} ", l), Style::default().fg(Color::DarkGray)),
            Span::styled("─".repeat(dw), Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        lines.push(Line::from(Span::styled("─".repeat(w), Style::default().fg(Color::DarkGray))));
    }
}

fn pad(s: &str, w: usize) -> String {
    if s.len() >= w { s[..w].to_string() } else { format!("{}{}", s, " ".repeat(w - s.len())) }
}

fn fmt_num(v: f64) -> String {
    if v >= 1_000_000.0 { format!("{:.1}M", v / 1_000_000.0) }
    else if v >= 1_000.0 { format!("{:.1}K", v / 1_000.0) }
    else { format!("{:.0}", v) }
}
