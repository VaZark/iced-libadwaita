# Notebook

GTK4: [`GtkNotebook`](https://docs.gtk.org/gtk4/class.Notebook.html)

## What it is

Tabbed container; tab position on any edge, scrollable tabs, popup menu, drag between notebooks.

## CSS nodes

```
notebook
├── header.top|bottom|left|right
│   ├── [<action widget>]
│   ├── tabs
│   │   ├── [arrow]
│   │   ├── tab[.reorderable-page]
│   │   ╰── [arrow]
│   ╰── [<action widget>]
╰── stack
```

## Style classes and states

`.frame`, positional classes on `header`, `.reorderable-page`, `.dnd`.

## Adwaita defers

Use `AdwTabBar` + `AdwTabView` / `AdwViewSwitcher`.

## iced mapping

`tab_bar`.
