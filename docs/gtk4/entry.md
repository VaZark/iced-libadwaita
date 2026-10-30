# Entry

GTK4: [`GtkEntry`](https://docs.gtk.org/gtk4/class.Entry.html)

## What it is

Single-line text entry; scrolls to keep the cursor visible; optional icons (left/right), progress fraction or pulse behind the text, password visibility.

## CSS nodes

```
entry[.flat][.warning][.error]
├── text[.readonly]
├── image.left
├── image.right
╰── [progress[.pulse]]
```

## Style classes and states

`.flat`, `.warning`, `.error`, `.readonly` on `text`. (GTK does not document `.success`; Adwaita adds `.success/.warning/.error` retinting.)

## Adwaita defers

Translucent foreground fill, 6px radius, accent focus ring; validation via `.success/.warning/.error` classes.

## iced mapping

`style::text_input::{default,error,embedded}`.
