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

## iced mapping

`ExpanderRow::new(title, expanded, on_toggle).push(row)`; stateless (app owns `expanded`).
