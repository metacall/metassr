# MetaSSR design system

The visual identity shared by the landing site and every MetaSSR app. Apply it
by default when building an app; only deviate when the user explicitly asks
for a different look (dark mode, their own brand colors, "make it look like
X") — the user's request wins.

Source of truth in the metacall/metassr repo: `site/src/styles/global.css`
and the "Design system" section of `site/README.md`. A verbatim copy of the
stylesheet ships alongside this document as `reference/global.css` — write it
into the app's `src/styles/global.css` (replacing the scaffolded one).

## Tokens

All values are CSS custom properties on `:root`. Components reference only
variables, so rebranding is just editing `:root`.

| Token | Value | Purpose |
| --- | --- | --- |
| `--color-canvas` | `#f4f1ea` | warm page background |
| `--color-ink` | `#1f2933` | primary text |
| `--color-ink-muted` | `#52616b` | secondary text |
| `--color-accent` | `#006d77` | teal accent: links, buttons, highlights |
| `--color-accent-strong` | `#00565e` | accent hover/pressed |
| `--color-accent-contrast` | `#ffffff` | text on accent |
| `--font-sans` | Inter, ui-sans-serif, system-ui, ... | body + headings |
| `--font-mono` | ui-monospace, SFMono-Regular, ... | code |
| `--radius-pill` | `999px` | pill buttons, pills |
| `--space-1` … `--space-7` | 4, 8, 12, 16, 24, 32, 48 px | spacing scale |
| `--content` | `820px` | max content width |

## Rules

- **Background**: flat, full-bleed `--color-canvas`. No cards, no borders, no
  chrome — content sits directly on the canvas.
- **Accents**: teal (`--color-accent`) for links, buttons, eyebrows, pills and
  highlighted table cells. Hover uses `--color-accent-strong`.
- **Type**: Inter with a system-sans fallback; code uses `--font-mono`. Hero
  headings are 26px; section titles 1.15rem; eyebrows and table headers are
  small, bold, uppercase with wide letter-spacing.
- **Layout**: content is centered and constrained to `--content` (820px).
  Full-page views use the `.page` pattern (centered flex column, `min-height:
  100vh`). Spacing comes from the `--space-*` scale only.
- **Buttons**: pill-shaped (`--radius-pill`), accent background, white text,
  bold, with a subtle hover darken and active press; `:focus-visible` gets a
  2px accent outline.

## Patterns

Reuse these classes from `reference/global.css` rather than inventing new
ones:

| Class | Use for |
| --- | --- |
| `.page` | full-page centered container |
| `.heroTitle` / `.heroLead` | page intro (26px / 1rem) |
| `.eyebrow` | small uppercase accent label |
| `.brandLink` / `.brandMark` | inline brand link with logo |
| `.button` | primary pill button |
| `.pill` | small accent status/tag |
| `.cta` | muted footer/CTA link |
| `.sectionTitle` / `.sectionLead` | section heading + lead |
| `.dataTable` (+ `.tableWrap`) | data tables; `.isAccent` highlights a cell |

For a component not covered here, compose it from the tokens above and keep
it referencing only variables.