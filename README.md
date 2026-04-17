# ccview

A TUI companion app for [Claude Code](https://claude.ai/claude-code). Gives AI output a visual layer.

Claude Code writes JSON files. ccview renders them — tables, charts, checklists, timelines, metrics — live in your terminal. No servers, no browsers, no config.

```
┌─ ccview ─────────────────────────────────────────────────────┐
│ ◆ ccview          │ Weekly Review — Apr 14                   │
│                   │                                          │
│ FINANCE           │ Net Worth Δ     Action Items   Subs      │
│   ★ Weekly Review │ +$7,534         2/7 done       $730/mo   │
│     March Report  │ this month                               │
│                   │                                          │
│ STOCKS            │ Priority Actions                         │
│     NVDA Research │ › ○ Audit AWS instances — $685/mo        │
│                   │   ✓ Fix Chase checking min balance       │
│ CI                │   ○ Cancel ChatGPT Plus ($20/mo)         │
│     Deploy v2.4   │   ○ Call GEICO for competing quote       │
│                   │                                          │
│                   │ ▸ Subscription Breakdown                 │
│                   │ ▸ Budget vs Actual (March)               │
│                   │                                          │
│                   │ Upcoming                                 │
│                   │ Apr 22 ● NOW Earnings                    │
│                   │        │ Q1 report — critical test       │
│                   │ May 7  ● VST Earnings                    │
│                   │        │ 206% EPS growth expected         │
│ 4 pages · / filter│ ↑↓ scroll · i interactive · esc back     │
└──────────────────────────────────────────────────────────────┘
```

## Install

**One-line install (macOS / Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/aravs16/ccview/main/install.sh | sh
```

This downloads the right binary for your platform, installs it to `/usr/local/bin`, creates `~/.ccview/pages/`, and sets up the Claude Code skill if CC is installed. No Rust required.

**Manual download:**

Go to [Releases](https://github.com/aravs16/ccview/releases), download the binary for your platform:

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `ccview-aarch64-apple-darwin` |
| macOS (Intel) | `ccview-x86_64-apple-darwin` |
| Linux (x86_64) | `ccview-x86_64-unknown-linux-gnu` |
| Linux (ARM) | `ccview-aarch64-unknown-linux-gnu` |
| Windows (x86_64) | `ccview-x86_64-pc-windows-msvc.exe` |

```bash
chmod +x ccview-* && sudo mv ccview-* /usr/local/bin/ccview
```

**From source (requires Rust):**

```bash
git clone https://github.com/aravs16/ccview.git
cd ccview
cargo install --path .
```

## How it works

```
Claude Code (writes JSON) → ~/.ccview/pages/ → ccview (renders live)
```

1. Claude Code skills write JSON files to `~/.ccview/pages/`
2. ccview watches the directory and renders pages in real-time
3. User interactions (checkbox toggles, section collapses) are stored separately so CC can overwrite pages without losing your state

That's it. No servers, no APIs, no databases, no config files.

## Quick start

```bash
# Launch ccview
ccview

# In another terminal, create a page
cat > ~/.ccview/pages/hello.json << 'EOF'
{
  "title": "Hello World",
  "created": "2026-04-16T10:00:00Z",
  "updated": "2026-04-16T10:00:00Z",
  "blocks": [
    {
      "type": "callout",
      "style": "success",
      "title": "It works",
      "content": "This page appeared in ccview without restarting."
    }
  ]
}
EOF
```

The page appears instantly in ccview's sidebar. No restart needed.

## Keyboard shortcuts

### Sidebar (page list)

| Key | Action |
|-----|--------|
| `j` / `↓` | Next page |
| `k` / `↑` | Previous page |
| `Enter` | View page content |
| `Tab` | Switch to content pane |
| `/` | Filter pages by name |
| `d` | Delete current page |
| `q` | Quit |

### Content (viewing a page)

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `i` | Enter interactive mode |
| `Esc` | Back to sidebar |
| `Tab` | Switch to sidebar |

### Interactive mode (checklists, sections)

| Key | Action |
|-----|--------|
| `j` / `↓` | Next interactive item |
| `k` / `↑` | Previous interactive item |
| `x` / `Space` / `Enter` | Toggle checkbox or collapse/expand section |
| `Esc` | Exit interactive mode |

## Page format

A page is a JSON file with a title and an array of blocks:

```json
{
  "title": "Page Title",
  "subtitle": "optional subtitle",
  "channel": "finance",
  "created": "2026-04-16T10:00:00Z",
  "updated": "2026-04-16T10:00:00Z",
  "pinned": false,
  "tags": ["tag1", "tag2"],
  "ttl": "7d",
  "blocks": [...]
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `title` | yes | Page title, shown in sidebar |
| `subtitle` | no | Secondary text |
| `channel` | no | Groups pages in sidebar (default: `_inbox`) |
| `created` | yes | ISO 8601 datetime |
| `updated` | yes | ISO 8601 datetime |
| `pinned` | no | Pinned pages show at top with ★ |
| `tags` | no | For filtering with `/` |
| `ttl` | no | Auto-expire: `1h`, `1d`, `7d`, `30d`, `forever` (default: `7d`) |

### Channels

Pages are organized into channels via subdirectories:

```
~/.ccview/pages/
├── finance/
│   ├── report-march.json
│   └── weekly-review.json
├── stocks/
│   └── nvda-research.json
├── ci/
│   └── deploy-status.json
└── _inbox/           ← default for uncategorized pages
```

The `channel` field in JSON is optional — if omitted, the subdirectory name is used.

## Block types

### `metrics` — KPI cards

```json
{
  "type": "metrics",
  "items": [
    { "label": "Revenue", "value": "$68B", "change": "+73%", "sentiment": "positive" }
  ]
}
```

`sentiment`: `"positive"` (green) · `"negative"` (red) · omit (white)

### `table` — Data table

```json
{
  "type": "table",
  "title": "Top Merchants",
  "columns": [
    { "key": "name", "label": "Name" },
    { "key": "amount", "label": "Amount" }
  ],
  "rows": [
    { "name": "AWS", "amount": "$1,369" }
  ]
}
```

### `chart` — Bar chart

```json
{
  "type": "chart",
  "title": "Monthly Trend",
  "chart": "bar",
  "data": [
    { "label": "Jan", "value": 26784 },
    { "label": "Feb", "value": 25026 }
  ]
}
```

### `markdown` — Rich text

```json
{ "type": "markdown", "content": "## Title\n\nSome **bold** text.\n\n- Bullet one\n- Bullet two" }
```

### `callout` — Alert box

```json
{ "type": "callout", "style": "warning", "title": "Alert", "content": "Something needs attention." }
```

`style`: `"info"` · `"success"` · `"warning"` · `"error"`

### `timeline` — Ordered events

```json
{
  "type": "timeline",
  "title": "Upcoming",
  "items": [
    { "date": "May 20", "title": "NVDA Earnings", "description": "Q1 FY2027", "color": "green" }
  ]
}
```

### `kv` — Key-value pairs

```json
{ "type": "kv", "title": "Details", "items": [{ "key": "Price", "value": "$196" }] }
```

### `code` — Code block

```json
{ "type": "code", "language": "python", "content": "print('hello')" }
```

### `progress` — Progress bars

```json
{
  "type": "progress",
  "items": [
    { "label": "Budget", "value": 397, "max": 500, "color": "green" }
  ]
}
```

### `list` — Bullet list or interactive checklist

```json
{
  "type": "list",
  "id": "my-tasks",
  "title": "Tasks",
  "style": "checklist",
  "items": [
    { "text": "Do the thing", "checked": false },
    { "text": "Already done", "checked": true }
  ]
}
```

Checklists are interactive — press `i` then `x` to toggle items. State persists across page updates.

### `section` — Collapsible group

```json
{
  "type": "section",
  "id": "details",
  "title": "Details",
  "collapsed": true,
  "blocks": [...]
}
```

Press `Enter` on a focused section to expand/collapse. Nested blocks render inside.

### `divider` — Separator

```json
{ "type": "divider", "label": "Optional Label" }
```

## Interactive state

User interactions are stored separately from page content:

```
~/.ccview/
├── pages/          ← CC writes here (content)
│   └── finance/
│       └── review.json
└── state/          ← ccview writes here (user state)
    └── finance/
        └── review.state.json
```

When CC overwrites a page, your checkbox states and collapse preferences survive. The state file looks like:

```json
{
  "checks": {
    "my-tasks": { "0": true, "2": true }
  },
  "collapsed": {
    "details": false
  },
  "scroll": 12
}
```

Blocks use the `id` field as the state key. If no `id` is set, the block index (`block-0`, `block-1`, ...) is used. For stable state across page regenerations, always set explicit `id` values on interactive blocks.

## Claude Code integration

ccview ships with a skill file that teaches Claude Code the page format. After installing ccview, copy the skill:

```bash
mkdir -p ~/.claude/skills/view
cp skill/SKILL.md ~/.claude/skills/view/SKILL.md
```

Now Claude Code knows how to write ccview pages. You can:

- Ask CC to "publish that to ccview"
- Run `/view` after any skill
- Add ccview output to your own custom skills

### Adding ccview output to an existing skill

Add this to your skill's SKILL.md:

```markdown
## Visual Output

If ~/.ccview/pages/ exists, also publish results as a ccview page.
Write to ~/.ccview/pages/<channel>/<page-id>.json using the ccview block format.
```

CC reads this and knows to write the JSON alongside its normal output.

## Architecture

```
src/
├── main.rs       ← TUI app loop, keyboard handling, layout
├── protocol.rs   ← Page/Block type definitions (serde)
├── loader.rs     ← File discovery, channel grouping
├── render.rs     ← Block → TUI line rendering
└── state.rs      ← User interaction state (read/write)
```

**~600 lines of Rust.** No async runtime, no networking, no database. Reads files, renders them, writes state.

## Roadmap

- [ ] `--web` flag for rich HTML view on localhost
- [ ] `--export` for standalone HTML file output
- [ ] Table sorting (click column headers)
- [ ] Page linking (`{ "type": "link", "page": "other-page" }`)
- [ ] Page history (keep last N versions on overwrite)
- [ ] `brew install ccview`
- [ ] `cargo install ccview` (publish to crates.io)

## License

MIT
