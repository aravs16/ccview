# ccview

A TUI companion for [Claude Code](https://code.claude.com/docs/en/overview). Gives AI output a visual layer.

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

## Get Started

### 1. Install

```bash
curl -fsSL https://raw.githubusercontent.com/aravs16/ccview/main/install.sh | sh
```

This installs the binary and sets up Claude Code integration automatically. No Rust needed.

Or download a binary from [Releases](https://github.com/aravs16/ccview/releases):

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `ccview-aarch64-apple-darwin` |
| macOS (Intel) | `ccview-x86_64-apple-darwin` |
| Linux (x86_64) | `ccview-x86_64-unknown-linux-gnu` |
| Linux (ARM) | `ccview-aarch64-unknown-linux-gnu` |
| Windows (x86_64) | `ccview-x86_64-pc-windows-msvc.exe` |

### 2. Use with Claude Code

The installer copies a skill file to `~/.claude/skills/view/`. This teaches Claude Code the ccview format. Now you can:

**Ask Claude Code directly:**
```
> analyze my spending and publish to ccview
> create a ccview page comparing NVDA vs AVGO
```

**Use the `/view` slash command** after any skill:
```
> /financial-analyst 2026-03
> /view
```

**Add ccview output to your own skills** — add this to any SKILL.md:
```markdown
If ~/.ccview/pages/ exists, also publish results as a ccview page.
Write to ~/.ccview/pages/<channel>/<page-id>.json using the ccview block format.
```

### 3. Launch the viewer

```bash
ccview
```

Pages appear in the sidebar as Claude Code writes them. Live — no restart needed.

### 4. Navigate

| Sidebar | Content | Interactive |
|---------|---------|------------|
| `j/k` navigate | `j/k` scroll | `j/k` move between items |
| `Enter` open page | `i` enter interactive mode | `x` toggle checkbox |
| `/` filter | `Esc` back to sidebar | `Enter` expand/collapse section |
| `d` delete | `Tab` switch pane | `Esc` exit interactive mode |
| `q` quit | | |

---

## How It Works

```
Claude Code (writes JSON) → ~/.ccview/pages/ → ccview (renders live)
```

CC writes a JSON file. ccview watches the directory and renders it instantly. User interactions (checkbox toggles, section collapses) are stored separately in `~/.ccview/state/` — CC can overwrite pages without losing your state.

No servers, no APIs, no databases, no config files.

## Page Format

A page is a JSON file with a title and an array of blocks:

```json
{
  "title": "Monthly Report",
  "subtitle": "/financial-analyst",
  "channel": "finance",
  "created": "2026-04-16T10:00:00Z",
  "updated": "2026-04-16T10:00:00Z",
  "pinned": false,
  "ttl": "7d",
  "blocks": [
    { "type": "metrics", "items": [{ "label": "Revenue", "value": "$68B", "sentiment": "positive" }] },
    { "type": "table", "title": "Top Items", "columns": ["Name", "Amount"], "rows": [{"name": "AWS", "amount": "$1,369"}] },
    { "type": "callout", "style": "warning", "title": "Alert", "content": "AWS bill spiked 37%." }
  ]
}
```

Pages are organized into **channels** via subdirectories:

```
~/.ccview/pages/
├── finance/          ← channel
│   └── report.json
├── stocks/
│   └── nvda.json
└── _inbox/           ← default
```

## Block Types

12 block types cover most structured output:

| Type | What it renders | Example use |
|------|----------------|-------------|
| `metrics` | KPI cards in a row | Revenue, EPS, build status |
| `table` | Data table | Top merchants, test results |
| `chart` | Bar chart | Monthly trends, comparisons |
| `markdown` | Rich text | Analysis, explanations |
| `callout` | Alert box (info/success/warning/error) | Flags, alerts |
| `timeline` | Vertical event timeline | Earnings dates, catalysts |
| `kv` | Key-value pairs | Stock details, config |
| `code` | Syntax-highlighted code | Diffs, commands |
| `progress` | Progress bars | Budgets, goals |
| `list` | Bullet list or interactive checklist | Action items |
| `section` | Collapsible group of blocks | Details, breakdowns |
| `divider` | Separator with optional label | Section breaks |

<details>
<summary>Full block reference with JSON examples</summary>

### metrics

```json
{
  "type": "metrics",
  "items": [
    { "label": "Revenue", "value": "$68B", "change": "+73%", "sentiment": "positive" },
    { "label": "EPS", "value": "$1.62", "sentiment": "positive" }
  ]
}
```

`sentiment`: `"positive"` (green) / `"negative"` (red) / omit (white)

### table

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

### chart

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

### markdown

```json
{ "type": "markdown", "content": "## Findings\n\nYour **AWS bill** is out of control.\n\n- Cancel ChatGPT\n- Fix Chase fee" }
```

### callout

```json
{ "type": "callout", "style": "warning", "title": "Cost Alert", "content": "AWS bill increased 37%." }
```

Styles: `info` / `success` / `warning` / `error`

### timeline

```json
{
  "type": "timeline",
  "title": "Catalysts",
  "items": [
    { "date": "May 20", "title": "NVDA Earnings", "description": "Q1 FY2027", "color": "green" }
  ]
}
```

### kv

```json
{ "type": "kv", "title": "Details", "items": [{ "key": "Price", "value": "$196.51" }] }
```

### code

```json
{ "type": "code", "language": "python", "content": "print('hello')" }
```

### progress

```json
{
  "type": "progress",
  "items": [
    { "label": "Dining Budget", "value": 397, "max": 500, "color": "green" },
    { "label": "Shopping", "value": 680, "max": 500, "color": "red" }
  ]
}
```

### list (interactive)

```json
{
  "type": "list",
  "id": "tasks",
  "title": "Action Items",
  "style": "checklist",
  "items": [
    { "text": "Cancel ChatGPT", "checked": false },
    { "text": "Fix Chase fee", "checked": true }
  ]
}
```

Press `i` then `x` to toggle checkboxes. State persists even when CC rewrites the page.

### section (collapsible)

```json
{
  "type": "section",
  "id": "details",
  "title": "Details",
  "collapsed": true,
  "blocks": [ ...nested blocks... ]
}
```

### divider

```json
{ "type": "divider", "label": "Optional Label" }
```

</details>

## Interactive State

User state is stored separately from page content:

```
~/.ccview/
├── pages/    ← Claude Code writes here (content)
└── state/    ← ccview writes here (checkboxes, collapses, scroll)
```

CC overwrites a page → your checkbox states survive. Set `"id"` on interactive blocks for stable state across regenerations.

## For Skill Authors

Want your Claude Code skill to produce ccview output? Two options:

**Option A:** Tell users to run `/view` after your skill.

**Option B:** Add ccview output directly to your skill. Add this to your SKILL.md:

```markdown
## Visual Output

If ~/.ccview/pages/ exists, write results to ~/.ccview/pages/<channel>/<id>.json:
- metrics block for summary KPIs
- table block for tabular data
- callout block for alerts
- list block with checklist style for action items
```

The full block schema is in `skill/SKILL.md` in this repo.

## Build from Source

```bash
git clone https://github.com/aravs16/ccview.git
cd ccview
cargo install --path .
```

## Architecture

```
src/
├── main.rs       ← TUI app, keyboard handling, layout
├── protocol.rs   ← Page/Block type definitions
├── loader.rs     ← File discovery, channel grouping
├── render.rs     ← Block → terminal rendering
└── state.rs      ← User interaction persistence
```

~850 lines of Rust. 1.4MB binary. <10ms startup.

## Roadmap

- [ ] `--web` flag for rich HTML view on localhost
- [ ] `--export` for standalone HTML output
- [ ] Table sorting
- [ ] Page linking
- [ ] Page version history
- [ ] `brew install ccview`
- [ ] Publish to crates.io

## License

MIT
