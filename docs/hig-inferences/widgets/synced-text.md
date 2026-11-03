# SyncedText

No libadwaita or GTK counterpart. Lyrics, subtitles, a transcript or a changelog timeline: lines that carry a time.

## Anatomy and behavior

- **Lines.** `primitives::synced::Line { at, text }`, ordered by time. `synced::current(lines, pos)` is the last line that started (none before the first).
- **Display.** The current line is large; the others are dimmed. Pressing a line asks to seek to its time (`on_seek(at)`). An empty list shows a placeholder.
- **Scrolling.** The app owns the position. When the current line changes it runs `follow_synced_text(id, current, total)`, which scrolls the list.

## iced mapping

A `scrollable` of flat buttons. `synced::scroll_to` gives the relative offset for `snap_to`.
