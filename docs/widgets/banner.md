# Banner

libadwaita: [`Banner`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Banner.html)

## What it is

A bar showing contextual information, hidden until `revealed`.

## Anatomy and behavior

Title centered or left depending on room; optional button (`button-style` gray or suggested, 1.7); `use-markup`.

## CSS nodes and classes

`banner`.

## Spec

Accent-filled bar with accent foreground, button right.

## iced mapping

`banner(title).button(label, msg).revealed(bool)`.
