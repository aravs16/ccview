---
name: view
description: Publish visual output to ccview — the Claude Code companion viewer. Use after any skill that produces structured results (reports, analysis, research, status).
allowed-tools:
  - "Bash"
---

# ccview — Visual Output

Publish structured visual output that the user can browse in the ccview TUI app.

## How it works

Write a JSON file to `~/.ccview/pages/`. The ccview app watches this directory and renders pages in real-time.

## When to use

After generating any structured output — financial reports, stock research, test results, data analysis, deploy status, code reviews — also write a ccview page so the user can view it visually.

**Check first:** Only write if `~/.ccview/pages/` exists. If it doesn't, skip silently.

## File format

```bash
cat > ~/.ccview/pages/<page-id>.json << 'EOF'
{
  "title": "Page Title",
  "subtitle": "source skill or context",
  "created": "<ISO 8601 timestamp>",
  "updated": "<ISO 8601 timestamp>",
  "pinned": false,
  "tags": ["tag1", "tag2"],
  "ttl": "7d",
  "blocks": [ ... ]
}
EOF
```

- `page-id`: kebab-case, descriptive (e.g. `financial-report-2026-03`, `nvda-research`)
- Writing the same id again overwrites (updates the page)
- `ttl`: `"1h"`, `"1d"`, `"7d"`, `"30d"`, or `"forever"` (default: `"7d"`)
- `pinned`: true = stays at top, never auto-expires

## Block types

Blocks render top-to-bottom in order. Use the right block for the data.

### metrics — KPI cards in a row

```json
{
  "type": "metrics",
  "items": [
    { "label": "Revenue", "value": "$68B", "change": "+73%", "sentiment": "positive" },
    { "label": "EPS", "value": "$1.62", "change": "+82%", "sentiment": "positive" }
  ]
}
```

`sentiment`: `"positive"` (green), `"negative"` (red), or omit (white).

### table — Data table

```json
{
  "type": "table",
  "title": "Top Merchants",
  "columns": [
    { "key": "name", "label": "Name" },
    { "key": "amount", "label": "Amount" }
  ],
  "rows": [
    { "name": "AWS", "amount": "$1,369" },
    { "name": "GEICO", "amount": "$457" }
  ]
}
```

Columns can be simplified: `"columns": ["Name", "Amount"]` (keys auto-derived as lowercase).

### chart — Bar chart

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

`chart`: `"bar"`, `"hbar"`, `"line"`, `"sparkline"`.

### markdown — Rich text

```json
{
  "type": "markdown",
  "content": "## Findings\n\nYour **AWS bill** is out of control.\n\n- Cancel ChatGPT\n- Fix Chase fee"
}
```

Supports headers (`#`, `##`), bold (`**text**`), lists (`-`).

### callout — Alert box

```json
{
  "type": "callout",
  "style": "warning",
  "title": "Cost Alert",
  "content": "AWS bill increased 37% this month."
}
```

`style`: `"info"`, `"success"`, `"warning"`, `"error"`.

### timeline — Ordered events

```json
{
  "type": "timeline",
  "title": "Catalysts",
  "items": [
    { "date": "May 20", "title": "NVDA Earnings", "description": "EPS $1.76 expected", "color": "green" },
    { "date": "Jun 4", "title": "AVGO Earnings", "color": "blue" }
  ]
}
```

### kv — Key-value pairs

```json
{
  "type": "kv",
  "title": "Details",
  "items": [
    { "key": "Price", "value": "$196.51" },
    { "key": "P/E", "value": "34x" }
  ]
}
```

### code — Code block

```json
{
  "type": "code",
  "title": "Fix",
  "language": "python",
  "content": "def calc(x):\n    return x * 0.22"
}
```

### progress — Progress bars

```json
{
  "type": "progress",
  "items": [
    { "label": "Dining Budget", "value": 397, "max": 500, "color": "green" },
    { "label": "Shopping", "value": 680, "max": 500, "color": "red" }
  ]
}
```

### list — Bullet or checklist

```json
{
  "type": "list",
  "title": "Action Items",
  "style": "checklist",
  "items": [
    { "text": "Cancel ChatGPT", "checked": false },
    { "text": "Fix Chase fee", "checked": true }
  ]
}
```

`style`: `"bullet"` or `"checklist"`. For bullet, items can be plain strings: `"items": ["Item 1", "Item 2"]`.

### divider — Separator

```json
{ "type": "divider", "label": "Optional Section Label" }
```

### section — Collapsible group

```json
{
  "type": "section",
  "title": "Details",
  "collapsed": true,
  "blocks": [ ... nested blocks ... ]
}
```

## Guidelines

- Use **metrics** for the top-level summary (3-5 KPIs)
- Use **callout** to highlight problems or wins
- Use **table** for any list of items with multiple fields
- Use **chart** for trends over time
- Use **timeline** for upcoming events with dates
- Use **kv** for entity details (stock info, config, etc.)
- Use **list** with `"checklist"` style for action items
- Keep pages focused — one topic per page
- Set `"pinned": true` for dashboards, `"ttl": "1d"` for ephemeral results
- Always set `created` and `updated` to current ISO 8601 datetime
