# SearchBar

GTK4: [`GtkSearchBar`](https://docs.gtk.org/gtk4/class.SearchBar.html)

## What it is

Reveals a search entry (typing or toggle), can hold extra widgets; Esc hides; needs a key-capture widget for type-to-search.

## CSS nodes

```
searchbar
╰── revealer
    ╰── box
        ├── [child]
        ╰── [button.close]
```

## Style classes and states

`.close` on the optional button; libadwaita `.inline` for a neutral look.

## iced mapping

_Not implemented._
