# ButtonRow

libadwaita: [`ButtonRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ButtonRow.html)

## What it is

A list row that looks like a button; for actions at the end of a boxed list (e.g. Delete).

## Anatomy and behavior

Title with optional start and end icons, always activatable. `.suggested-action` / `.destructive-action` supported. Since 1.6.

## CSS nodes and classes

`row.button` > `box` > `image.icon.start`, `label.title`, `image.icon.end`.

## Spec

Content centered; accent/destructive colors the title (standalone color) rather than filling.

## iced mapping

`ButtonRow::new(title).role(Role::Destructive).on_activate(msg)`.
