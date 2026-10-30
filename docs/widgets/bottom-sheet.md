# BottomSheet

libadwaita: [`BottomSheet`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.BottomSheet.html)

## What it is

Bottom sheet with optional bottom bar integrated with content (1.6); for transient sheets use Dialog.

## Anatomy and behavior

`open`, `full-width` (default true), `align`, `modal` (default true: dims content), `show-drag-handle` (default true), `reveal-bottom-bar`; sheet follows its content's natural height.

## iced mapping

`bottom_sheet(content, sheet, open, on_close)`.

## Deviations / gaps

No bottom bar, drag.
