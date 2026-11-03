# ExpanderRow

libadwaita: [`ExpanderRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ExpanderRow.html)

## What it is

A row that reveals nested rows below it.

## Anatomy and behavior

- `expanded`, `enable-expansion`, optional enable switch (`show-enable-switch`), prefix/suffix, nested `add_row`.
- Header is an action row with an arrow (`expander-row-arrow`) that rotates when expanded.
- `title-lines`/`subtitle-lines` as ActionRow. `.property` supported.

## CSS nodes and classes

`row.expander` (`.empty` without children) > `row.header`, `list.nested`, `image.expander-row-arrow`.

## Spec

Nested list sits on a slightly shaded background separated by a hairline ‡.

## When to use TreeRow instead

`ExpanderRow` is libadwaita's: the whole header toggles, and it is for one level of extra detail. For a **hierarchy** (nested groups, a selectable title next to the arrow, children that are rows or any element) use [TreeRow](tree-row.md), which is for nesting within a page, not for navigation across pages.

## Indent

`ExpanderRow::indent(px)` offsets the nested rows. It is a plain value that **starts at the toolkit's default**, `Metrics::expander_indent`, which the desktop config sets with `[rows] expander-indent` (24 px, the width of the arrow handle); calling `.indent(px)` overrides it for that row. libadwaita nests flush, so **0 restores it**. The hairlines still span the whole list; only the rows move.

## iced mapping

`ExpanderRow::new(title, expanded, on_toggle).push(row)`; stateless (app owns `expanded`). The nested list is always mounted inside a `reveal` that slides it open/closed (250ms, `ADW_EASE`).
