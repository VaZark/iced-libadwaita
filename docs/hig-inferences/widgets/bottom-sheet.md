# BottomSheet

libadwaita: [`BottomSheet`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.BottomSheet.html)

## What it is

Bottom sheet with optional bottom bar integrated with content (1.6); for transient sheets use Dialog.

## Anatomy and behavior

`open`, `full-width` (default true), `align`, `modal` (default true: dims content), `show-drag-handle` (default true), `reveal-bottom-bar`; sheet follows its content's natural height.

## iced mapping

`bottom_sheet(content, sheet, open, on_close)` / `bottom_sheet_in_window(.., radius)`; slides up on the sheet spring (damping ratio 0.8, mass 1, stiffness 400) with a fading scrim.

## Deviations / gaps

No bottom bar, drag, or slide-out (it animates in only).
