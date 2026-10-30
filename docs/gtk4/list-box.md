# ListBox

GTK4: [`GtkListBox`](https://docs.gtk.org/gtk4/class.ListBox.html)

## What it is

Vertical list of `GtkListBoxRow`s with selection, activation, headers, separators, sort/filter, placeholder.

## CSS nodes

```
list[.separators][.rich-list][.navigation-sidebar][.boxed-list]
╰── row[.activatable]
```

## Style classes and states

`.separators`, `.boxed-list`, `.rich-list`, `.navigation-sidebar`, `.data-table`, row `.activatable`.

## Adwaita defers

`.boxed-list` (one card, separators from `--card-shade-color`), `.boxed-list-separate`, `.navigation-sidebar` (neutral selection). Deprecated: `.content`, `.sidebar`.

## iced mapping

`boxed_list`, `navigation_sidebar`, `ActionRow` and friends.
