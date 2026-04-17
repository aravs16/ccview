mod loader;
mod protocol;
mod render;
mod state;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use loader::{get_channels, load_all_pages};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use protocol::LoadedPage;
use render::{FocusKind, RenderResult};
use state::PageState;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;
use std::sync::mpsc;
use std::time::Duration;

#[derive(PartialEq)]
enum Focus { Sidebar, Content }

struct App {
    pages: Vec<LoadedPage>,
    channels: Vec<String>,
    selected: usize,
    scroll: u16,
    focus: Focus,
    should_quit: bool,
    filter: String,
    filtering: bool,
    // Interactive content state
    page_state: PageState,
    focused_item: Option<usize>, // index into focus_items
    last_render: Option<RenderResult>,
    // Channel collapse state
    collapsed_channels: std::collections::HashSet<String>,
}

impl App {
    fn new() -> Self {
        let pages = load_all_pages();
        let channels = get_channels(&pages);
        let page_state = pages.first()
            .map(|p| PageState::load(&p.id, &p.channel))
            .unwrap_or_default();
        Self {
            pages, channels, selected: 0, scroll: 0,
            focus: Focus::Sidebar, should_quit: false,
            filter: String::new(), filtering: false,
            page_state, focused_item: None, last_render: None,
            collapsed_channels: std::collections::HashSet::new(),
        }
    }

    fn reload(&mut self) {
        let prev_id = self.pages.get(self.selected).map(|p| p.id.clone());
        self.pages = load_all_pages();
        self.channels = get_channels(&self.pages);
        if let Some(id) = prev_id {
            if let Some(pos) = self.pages.iter().position(|p| p.id == id) {
                self.selected = pos;
            }
        }
        if self.selected >= self.pages.len() && !self.pages.is_empty() {
            self.selected = self.pages.len() - 1;
        }
        self.load_state_for_selected();
    }

    fn load_state_for_selected(&mut self) {
        if let Some(page) = self.pages.get(self.selected) {
            self.page_state = PageState::load(&page.id, &page.channel);
            self.scroll = self.page_state.scroll;
            self.focused_item = None;
        }
    }

    fn save_state(&mut self) {
        if let Some(page) = self.pages.get(self.selected) {
            self.page_state.scroll = self.scroll;
            self.page_state.save(&page.id, &page.channel);
        }
    }

    fn filtered_pages(&self) -> Vec<(usize, &LoadedPage)> {
        let f = self.filter.to_lowercase();
        self.pages.iter().enumerate().filter(|(_, p)| {
            if !self.filter.is_empty() {
                return p.page.title.to_lowercase().contains(&f)
                    || p.page.tags.iter().any(|t| t.to_lowercase().contains(&f))
                    || p.id.to_lowercase().contains(&f);
            }
            // Hide pages in collapsed channels
            !self.collapsed_channels.contains(&p.channel)
        }).collect()
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if self.filtering {
            match code {
                KeyCode::Esc => { self.filtering = false; self.filter.clear(); }
                KeyCode::Enter => { self.filtering = false; }
                KeyCode::Backspace => { self.filter.pop(); }
                KeyCode::Char(c) => { self.filter.push(c); }
                _ => {}
            }
            return;
        }

        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => self.should_quit = true,
            KeyCode::Tab => {
                self.save_state();
                self.focus = if self.focus == Focus::Sidebar { Focus::Content } else { Focus::Sidebar };
                if self.focus == Focus::Content { self.focused_item = None; }
            }
            _ => {}
        }

        match self.focus {
            Focus::Sidebar => match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.save_state();
                        self.selected -= 1;
                        self.load_state_for_selected();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let filtered = self.filtered_pages();
                    if self.selected + 1 < filtered.len() {
                        self.save_state();
                        self.selected += 1;
                        self.load_state_for_selected();
                    }
                }
                KeyCode::Enter => {
                    self.focus = Focus::Content;
                    self.focused_item = None;
                }
                KeyCode::Char('/') => { self.filtering = true; self.filter.clear(); }
                KeyCode::Char('d') => {
                    if let Some(page) = self.pages.get(self.selected) {
                        // Try channel subdir first, then root
                        let channel_path = loader::pages_dir().join(&page.channel).join(format!("{}.json", page.id));
                        let root_path = loader::pages_dir().join(format!("{}.json", page.id));
                        if channel_path.exists() { std::fs::remove_file(channel_path).ok(); }
                        else { std::fs::remove_file(root_path).ok(); }
                        self.reload();
                    }
                }
                _ => {}
            },
            Focus::Content => match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(fi) = self.focused_item {
                        if fi > 0 { self.focused_item = Some(fi - 1); }
                        else { self.focused_item = None; }
                    } else {
                        self.scroll = self.scroll.saturating_sub(3);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max_focus = self.last_render.as_ref().map(|r| r.focus_items.len()).unwrap_or(0);
                    if let Some(fi) = self.focused_item {
                        if fi + 1 < max_focus { self.focused_item = Some(fi + 1); }
                    } else {
                        self.scroll = self.scroll.saturating_add(3);
                    }
                }
                KeyCode::Char('x') | KeyCode::Char(' ') => {
                    // Toggle focused item
                    if let Some(fi) = self.focused_item {
                        if let Some(render) = &self.last_render {
                            if let Some(item) = render.focus_items.get(fi) {
                                match &item.kind {
                                    FocusKind::CheckItem(idx) => {
                                        self.page_state.toggle_check(&item.block_id, *idx);
                                        self.save_state();
                                    }
                                    FocusKind::SectionToggle => {
                                        self.page_state.toggle_collapsed(&item.block_id);
                                        self.save_state();
                                    }
                                    FocusKind::TableSort(col) => {
                                        self.page_state.cycle_sort(&item.block_id, col);
                                        self.save_state();
                                    }
                                }
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    // Same as x/space for interactive items
                    if let Some(fi) = self.focused_item {
                        if let Some(render) = &self.last_render {
                            if let Some(item) = render.focus_items.get(fi) {
                                match &item.kind {
                                    FocusKind::CheckItem(idx) => {
                                        self.page_state.toggle_check(&item.block_id, *idx);
                                        self.save_state();
                                    }
                                    FocusKind::SectionToggle => {
                                        self.page_state.toggle_collapsed(&item.block_id);
                                        self.save_state();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('i') => {
                    // Enter interactive mode (start focusing items)
                    let has_items = self.last_render.as_ref().map(|r| !r.focus_items.is_empty()).unwrap_or(false);
                    if has_items {
                        self.focused_item = Some(0);
                    }
                }
                KeyCode::Esc => {
                    if self.focused_item.is_some() {
                        self.focused_item = None;
                    } else {
                        self.save_state();
                        self.focus = Focus::Sidebar;
                    }
                }
                _ => {}
            },
        }
    }
}

fn main() -> io::Result<()> {
    loader::ensure_dir();

    let (tx, rx) = mpsc::channel();
    let mut watcher = recommended_watcher(move |_res| { tx.send(()).ok(); })
        .expect("Failed to create file watcher");
    // Watch root and subdirs
    watcher.watch(&loader::pages_dir(), RecursiveMode::Recursive).ok();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        if rx.try_recv().is_ok() {
            while rx.try_recv().is_ok() {}
            app.reload();
        }

        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code, key.modifiers);
            }
        }

        if app.should_quit { break; }
    }

    app.save_state();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(40)])
        .split(f.area());

    render_sidebar(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
}

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(Color::Cyan).bold()),
        Span::styled("ccview", Style::default().fg(Color::White).bold()),
    ]))
    .block(Block::default().borders(Borders::BOTTOM | Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(title, chunks[0]);

    // Build sidebar items grouped by channel
    let filtered = app.filtered_pages();
    let mut items: Vec<ratatui::widgets::ListItem> = Vec::new();
    let mut current_channel = String::new();
    let mut page_idx = 0;

    for (_, page) in &filtered {
        if page.channel != current_channel {
            current_channel = page.channel.clone();
            let ch_label = if current_channel == "_inbox" { "inbox" } else { &current_channel };
            items.push(ratatui::widgets::ListItem::new(Line::from(Span::styled(
                format!(" {} ", ch_label.to_uppercase()),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
            ))));
        }

        let pin = if page.page.pinned { "★ " } else { "  " };
        let style = if page_idx == app.selected && app.focus == Focus::Sidebar {
            Style::default().fg(Color::Black).bg(Color::White)
        } else if page_idx == app.selected {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        let title = &page.page.title;
        let max_w = 25;
        let display = format!("  {}{}", pin, if title.len() > max_w { &title[..max_w] } else { title });
        items.push(ratatui::widgets::ListItem::new(Line::from(Span::styled(display, style))));
        page_idx += 1;
    }

    let list = List::new(items).block(
        Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(list, chunks[1]);

    let footer_content = if app.filtering {
        Line::from(vec![
            Span::styled(" /", Style::default().fg(Color::Cyan)),
            Span::styled(&app.filter, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(Span::styled(
            format!(" {} pages · / filter · q quit", app.pages.len()),
            Style::default().fg(Color::DarkGray),
        ))
    };

    let footer = Paragraph::new(footer_content).block(
        Block::default().borders(Borders::TOP | Borders::RIGHT).border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[2]);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    let filtered = app.filtered_pages();

    if filtered.is_empty() {
        let empty = Paragraph::new(vec![
            Line::raw(""),
            Line::from(Span::styled("  No pages yet", Style::default().fg(Color::DarkGray))),
            Line::raw(""),
            Line::from(Span::styled("  Write JSON to ~/.ccview/pages/<channel>/", Style::default().fg(Color::DarkGray))),
        ]);
        f.render_widget(empty, area);
        return;
    }

    let (_, page) = match filtered.get(app.selected) {
        Some(p) => p,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    // Header
    let mut title_spans = vec![
        Span::styled(format!(" {}", page.page.title), Style::default().fg(Color::White).bold()),
    ];
    if let Some(sub) = &page.page.subtitle {
        title_spans.push(Span::styled(format!("  {}", sub), Style::default().fg(Color::DarkGray)));
    }
    if page.channel != "_inbox" {
        title_spans.push(Span::styled(format!("  #{}", page.channel), Style::default().fg(Color::Cyan)));
    }

    let header = Paragraph::new(Line::from(title_spans)).block(
        Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(header, chunks[0]);

    // Render blocks with state
    let content_width = chunks[1].width;
    let result = render::render_blocks(&page.page.blocks, content_width, &app.page_state, app.focused_item);

    let scroll = app.scroll as usize;
    let visible_height = chunks[1].height as usize;
    let display_lines: Vec<Line> = result.lines.iter()
        .skip(scroll)
        .take(visible_height)
        .cloned()
        .collect();

    app.last_render = Some(result);

    let content = Paragraph::new(display_lines).block(Block::default().padding(Padding::new(2, 2, 1, 0)));
    f.render_widget(content, chunks[1]);

    // Interactive footer
    let hint = if app.focused_item.is_some() {
        " ↑↓ navigate · x toggle · esc unfocus"
    } else if app.focus == Focus::Content {
        " ↑↓ scroll · i interactive · esc back"
    } else {
        " enter view · tab switch"
    };
    let footer = Paragraph::new(Line::from(Span::styled(
        hint.to_string(),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(footer, chunks[2]);
}
