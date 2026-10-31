# TextView

GTK4: [`GtkTextView`](https://docs.gtk.org/gtk4/class.TextView.html)

## What it is

Multi-line text display/editor over a `GtkTextBuffer`; undo/redo, clipboard actions, context menu.

## CSS nodes

```
textview.view
├── border.top|left|right|bottom
├── text
│   ├── [preedit…]
│   ╰── [selection]
╰── [window.popup]
```

## Style classes and states

`.view` plus `border.*` and preedit roles.

## Adwaita defers

Use `.view` colors; libadwaita lets a TextView sit in a card with `.inline`.

## iced mapping

`style::text_editor::default` (stock iced `text_editor`).
