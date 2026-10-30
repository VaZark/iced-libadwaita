# PasswordEntry

GTK4: [`GtkPasswordEntry`](https://docs.gtk.org/gtk4/class.PasswordEntry.html)

## What it is

Hides contents, blocks clipboard copy, warns on Caps Lock, optional reveal (peek) icon.

## CSS nodes

```
entry.password
╰── text
    ╰── image.caps-lock-indicator
```

## Style classes and states

`.password`, `.caps-lock-indicator`

## Adwaita defers

`AdwPasswordEntryRow` is the Adwaita form.

## iced mapping

`EntryRow::password(true)` (no reveal, no Caps Lock indicator).
