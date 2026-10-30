# PathBar (Nautilus-style)

libadwaita: [`HeaderBar`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.HeaderBar.html)

## What it is

Breadcrumb navigation used in place of a header title in file managers like Nautilus. Not a libadwaita widget; documented because it is the canonical example of a replaced header title.

## Anatomy and behavior

Segments for each folder, the current one highlighted; selecting one navigates there. The first segment is the Home icon. It replaces the title, so the header bar has no title text; window buttons are unaffected. Pairs with a flat (continuous) header.

## iced mapping

`path_bar(segments, current, on_select)` builds a `ToggleGroup`; use it as `HeaderBar::title_widget`.

## Deviations / gaps

No collapsing of leading segments when narrow (the title slot clips instead), no editable location entry.
