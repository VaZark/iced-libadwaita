# Label

GTK4: [`GtkLabel`](https://docs.gtk.org/gtk4/class.Label.html)

## What it is

Displays text; wrap, ellipsize, `lines`, `width-chars`/`max-width-chars`, selectable, links, natural wrap mode.

## Behavior

- A wrapping label rewraps to all available width.
- An ellipsized label's natural width is the full text; `width-chars`/`max-width-chars` set its minimum.
- iced cannot ellipsize: we wrap (rows) or never wrap and clip (`typography::label`).

## CSS nodes

```
label
├── [selection]
╰── [link]
```

## Style classes and states

`.link`; legacy `.title`, `.subtitle`, `.dim-label`; `.keycap` in shortcuts.

## Adwaita defers

Prefer `.title-N`, `.heading`, `.caption`, `.dimmed`, `.numeric`, `.monospace` over `.title/.subtitle/.dim-label` (deprecated).

## iced mapping

`widget::typography::*`.
