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
- Navigation density: `sm`, `md`, `lg` variants with per-key inherited geometry;
  local `SidebarPadding` helpers resolve width rules. Existing navigation builders
  still need explicit plumbing to use those local results throughout.
- `Tabs`: named `default`, `outline`, `pills`, `floating` presentations, using their
  existing renderers rather than pretending their structural differences are colors.
- `ToolbarView`: `flat`, `raised`, `raised-border` presentations.

Tree, TreeRow, Tabs and Toolbar accept local token/config snapshots so previews
need not install global configuration. Geometry and draw-time theme colors are
separate: pass the matching metrics and theme too.

The editor exposes structured rule rows, registered variant choices, a layout-width
slider, and local-token previews. Light/dark selection is not widget variant selection.

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
