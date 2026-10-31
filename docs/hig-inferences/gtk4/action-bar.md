# ActionBar

GTK4: [`GtkActionBar`](https://docs.gtk.org/gtk4/class.ActionBar.html)

## What it is

Contextual actions bar shown below content; start/end areas plus a centered widget that stays centered across the full width; `revealed`.

## CSS nodes

```
actionbar
╰── revealer
    ╰── box
        ├── box.start
        ├── [center widget]
        ╰── box.end
```

## Style classes and states

`.start`, `.end`.

## Adwaita defers

Same look as a header bar (`.toolbar`); in Adwaita use a bottom bar in `AdwToolbarView`.

## iced mapping

`action_bar()` / `ToolbarView::bottom_bar`.
