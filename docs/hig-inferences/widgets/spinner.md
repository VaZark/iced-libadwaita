# Spinner

libadwaita: [`Spinner`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Spinner.html)

## What it is

Loading indicator; size follows available space between 16×16 and 64×64.

## Anatomy and behavior

For fixed size: center-align and set width/height request. Role: progress bar.

## CSS nodes and classes

Node `image.spinner`.

## Spec

Min 16, max 64.

## iced mapping

`spinner(size, rotation)` on a canvas; app drives `rotation` from `window::frames()`.
