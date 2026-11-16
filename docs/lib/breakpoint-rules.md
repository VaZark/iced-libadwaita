# Breakpoint rules and logical variants

A widget variant is a named set of optional typed overrides in `[widget.variant]`.
A breakpoint rule selects a variant; it does not introduce a second set of style
properties. Scheme variations remain orthogonal: `[dark:widget.variant]` overrides
that variant in the dark scheme.

```ini
[tabs]
breakpoint-rules=xs:pills,md:outline,lg:default

[tabs.outline]
indicator-height=2px
border-width=1px

[dark:tabs.outline]
border-width=2px

[toolbar]
breakpoint-rules=xs:flat,md:raised-border
```

Rules use comma-separated `breakpoint:variant` pairs. Classes are `xs`, `sm`, `md`,
`lg`, `xl`, `xxl`, using the configured breakpoint scale. The widest matching
minimum width wins; omitted classes inherit the preceding selection. Entry order
does not affect the result. Duplicate classes, malformed pairs, unknown classes,
and undeclared variant names invalidate the whole property. Config loading warns
and tries the lower layer; the editor reports the invalid value rather than
saving it. `breakpoint-rules` belongs to the base widget section, never a variant.

Explicit variant builders override rules. Explicit geometry builders override
variant geometry. A missing variant property inherits the base token, then the
widget's inherited metrics. Undeclared explicit variants use neutral/base geometry.
Custom variants must be declared with `Schema::variant`; putting an arbitrary
section in a file does not register executable behavior.

## Implemented consumers

- `Tree`: `.variant(name)`, tokenized row/indent/mark/connector/drag geometry;
  rules use available layout width. No responsive wrapper when rules are absent.
- `TreeRow`: `.variant(name)`, local tokens and `.available_width(width)`;
  `TreeRow::responsive(factory)` supplies layout width automatically.
- `SidebarList`: `sm`, `md`, `lg` and declared custom density variants. Rules use
  the list's available layout width automatically, including pinned rows, sliding
  pages and cards. `.variant(name)` or `.padding(level)` explicitly selects density;
  `.available_width(width)` supplies a known width without a responsive wrapper.
- `Tabs`: named `default`, `outline`, `pills`, `floating` presentations, using their
  existing renderers rather than pretending their structural differences are colors.
- `ToolbarView`: `flat`, `raised`, `raised-border` presentations.

Tree, TreeRow, SidebarList, Tabs and Toolbar accept local token/config snapshots so previews
need not install global configuration. Geometry and draw-time theme colors are
separate: pass the matching metrics and theme too.

The editor exposes structured rule rows, registered variant choices, a layout-width
slider, and local-token previews. Light/dark selection is not widget variant selection.

## Navigation density

```ini
[breakpoints]
md=700px

[navigation]
breakpoint-rules=xs:sm,md:lg
row-padding=12px
gap=2px
inset=6px

[navigation.sm]
row-height=30px

[navigation.lg]
row-height=48px
row-padding=14px
```

```rust,ignore
SidebarList::new(items)
    .theme_config(&preview_config) // Or .tokens(preview_config.tokens()).
    .metrics(preview_metrics)
```

The list snapshots its layout tokens locally; it does not install configuration.
The local `[breakpoints]` scale selects `lg` at 700px inclusively. Below that, `sm`
uses its own row height and inherits row padding, gap and inset separately from
`[navigation]`. Every name follows the same per-key path: declared variant key →
base navigation key → supplied metric. There is no size-name arithmetic or special
`sm`/`md`/`lg` renderer. An absent or opted-out variant inherits neutral base/metric
values, never another variant's geometry. Custom names must be registered in the
schema.

Density defaults are config-loaded. The shipped `themes/libadwaita.conf` explicitly
sets the existing libadwaita geometry:

| Config variant | Minimum row height | Horizontal row padding | Gap | Inset |
| --- | --- | --- | --- | --- |
| `navigation.sm` | 28px | 8px | 1px | 4px |
| `navigation.md` | 36px | 12px | 2px | 6px |
| `navigation.lg` | 44px | 14px | 2px | 6px |

These absolute values do not change through preset-specific arithmetic when
inherited metrics change. A theme can omit individual keys to inherit them instead.
`SidebarPadding::{Sm,Md,Lg}` are compatibility string IDs only, equivalent to
`.variant("sm")`, `.variant("md")`, or `.variant("lg")`; they neither define geometry
nor constrain custom variant names.

By default the width is the list's **own available width**, not the window width.
A responsive wrapper is added only when density rules or `.cards_below(class)`
need it. It rebuilds for size changes, not animation-only relayouts. When the parent
already knows the relevant width, pass `.available_width(width)`; the value is
resolved at render time, so its order relative to `.tokens()` does not matter.

Precedence is explicit and independent of rule order:

1. `.variant(name)` and `.padding(level)` are explicit density selections; the last
   call wins over rules. `.follow(...)` makes an explicit selection only when an
   adaptive layout is actually shown, preserving its shown-layout timing.
2. `.spacing(px)` and `.inset(px)` override the selected density's gap and inset.
3. Each item's `.padding(level)` overrides its row density, not the list gap/inset.
4. The item's typed `row_props` / `card_props` apply after resolved defaults.

All list rows use the same resolved minimum height and horizontal padding, and
all allow natural font growth above that minimum. No variant has a separate
content-height, vertical-padding, or font-growth rendering policy. Indentation, divider spacing and drag-edge
reach use the same local geometry, including explicit list gap/inset overrides.
Pinned rows and page rows resolve through the same context. Pages replace the
original items after `fixed_top`, rather than duplicating those items above a page.

Cards retain their subtitle, chevron, activation callback, single boxed surface and
hairline separators. Their minimum row height and horizontal padding now follow
sidebar density rather than unrelated action-row defaults. Subtitles can grow
beyond the minimum height. The resolved inset pads the page; the gap is distributed
above and below each card row, outside its hit target, within the boxed surface.

Density rules do **not** select list/cards, drawer/parent-page, collapsed/expanded
state, or a page. `.cards_below(class)` remains an explicit structural choice and
uses the same local breakpoint scale; `.style()` and `.follow()` otherwise retain
control of presentation. Split-view content-fit and drawer-width safety are
unchanged. In cards mode, pinned rows, pages and tree/drag affordances remain
list-only features, as before.

Limits: layout snapshots do not replace the draw-time `Theme` or typography; pass
matching theme/font settings for a faithful preview. The standalone `sidebar_row*`
and `navigation_sidebar*` functions resolve installed density tokens when built,
but accept already-built content and have no local snapshot or implicit width
plumbing. Use `SidebarList` for locally isolated, responsive destinations; a wrapper
cannot retroactively resize arbitrary caller-built rows. Split-view chrome geometry
is not a density variant consumer. This migration is covered by headless layout and
activation tests, not screenshot or full drag/navigation interaction acceptance.

## Logical-variant audit

These are source-audit findings, **not a claim that every item is migrated**.
Transient hover/pressed/focus state and ordinary data state are not logical variants.

| Layer / source | Logical choices | Treatment |
| --- | --- | --- |
| `components/avatar.rs` | circle, rounded, square | Radius/geometry policy; preserve square bounds for circles. |
| `components/badge.rs` | solid/tinted, chip size | Named surface and density variants. |
| `components/divider.rs` | solid, dashed, dotted; surface tone | Typed rendering policy, backed by its pattern renderer. |
| `components/entry.rs`, `widgets/entries/surface.rs` | flat/framed | Shared surface variants; password and multiline remain input semantics. |
| `components/input_wrapper.rs` | stacked/row | Structural adapter, retaining label/message placement behavior. |
| `widgets/navigation.rs` | list/cards, drawer/parent page, side-by-side/collapsed | Structural dispatch; preserve navigation state and content-fit constraints. |
| `widgets/responsive_table.rs` | table/cards | Structural dispatch; preserve column-fit safety. |
| `widgets/choice_row.rs` | toggles/dropdown | Structural dispatch; preserve calculated content fit. |
| `widgets/popover.rs`, `widgets/menu.rs` | popover/page/dialog, dropdown/full-page | Structural dispatch; preserve dismissal, scrim, focus and desktop-policy precedence. |
| `widgets/stepper.rs`, `widgets/pagination.rs` | size presets | Named density geometry; orientation/control presence remains structural. |
| `widgets/rich_text.rs`, `widgets/status_page.rs` | compact/regular | Typography and spacing variants. |
| `widgets/item_card.rs`, `widgets/item_row.rs` | compact/regular | Structural variants where pictures/columns change. |
| `widgets/carousel.rs` | dots/lines/none | Renderer, reserved geometry and hit regions must agree. |
| `widgets/toolbar.rs`, `widgets/tabs.rs` | named presentations above | Integrated consumers. |
| `primitives/bevel.rs` | bevel style, tones, edges | Rendering vocabulary; retain geometry implementation. |
| `primitives/responsive.rs`, `primitives/meta.rs` | `When`, triggers | Composition rules; existing first-match semantics differ from minimum-width cascading. |
| `components/classic*.rs`, `components/blocks.rs` | drawn marks, arrow scrollbars, block progress | Typed renderer policies; never sacrifice keyboard behavior for appearance. |

Tree drag/selection policy, calendar levels, scrub axes, range collision policy,
load triggers, and validation semantics are behavior, not cosmetic variants. A
named presentation can select an existing implementation, but cannot manufacture
that behavior through arbitrary style tokens.

## Remaining hardcoded-geometry audit candidates

Follow-up candidates outside the migrated tree/navigation/tab/toolbar and control
geometry include separator widths in dialogs, menus, tables, rows, split views and
section dividers; profile-header padding; rich-text indentation; drag insets;
resizable/reorderable surface geometry; icon-picker borders; and spoiler/table sizing
defaults. Review these against inherited metrics before adding tokens. Structural
zeros, normalized ratios and arithmetic derived from resolved geometry are not
independent design tokens.
