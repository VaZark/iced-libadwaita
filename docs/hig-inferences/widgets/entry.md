# Entry, text inputs (GtkEntry)

libadwaita: [`EntryRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.EntryRow.html)

## What it is

Not an Adw class. Translucent fill, 9px radius, focus ring in accent, `.success/.warning/.error` validation tints.

## Spec

min height 34 ‡, padding 9 horizontal ‡.

## iced mapping

`style::text_input::{default,error,embedded}`.

**Deviation (Mantine `leftSection` / `rightSection`).** GTK has only primary/secondary icons. `Entry::left_section` / `right_section` take any element (icon, spinner, button, badge, label), drawn in a transparent layer over the input so they sit inside its border and focus ring, vertically centered, `space_xs` from the edge. The text padding grows by the slot width (`primitives::sections::text_padding`) so text never overlaps; with no section the entry is unchanged. iced cannot measure a section before laying out the input, so the slot is `section_size` (icon + 2 × `space_xs`) unless `left_section_width` / `right_section_width` says otherwise. Sections are not dimmed automatically when the entry is read-only; `section_icon(..).disabled(true)` and `section_button(..).disabled(true)` dim them, and a button left enabled stays pressable. Helpers: `section_icon`, `section_button(..).active(bool)`.
