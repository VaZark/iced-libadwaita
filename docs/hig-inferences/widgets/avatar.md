# Avatar

libadwaita: [`Avatar`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Avatar.html)

## What it is

Round avatar: custom image, or initials (from `text`) on a color chosen from a hash of the text, or an icon when `show-initials` is false (`adw-avatar-default-symbolic`).

## Anatomy and behavior

`size`; initials from the text (first letters of first/last word).

## CSS nodes and classes

Node `avatar`.

## Spec

The hash/palette are not documented.

## iced mapping

`avatar(size, text)`; 12-color palette by hash; `.show_initials(false)`. No custom image.
