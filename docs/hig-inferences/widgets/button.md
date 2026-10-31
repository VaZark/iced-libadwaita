# Button (GtkButton) and style classes

libadwaita: [`Window`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Window.html)

## What it is

Not an Adw class; documented because every widget depends on it. See [style-classes](../design-language/style-classes.md).

## Anatomy and behavior

- Default: translucent fg fill (10%, hover 15%, active 30% ✓), 9px radius ✓, min 34 high ✓, padding 5×10 ✓.
- `.flat`, `.raised`, `.circular`, `.pill`, `.suggested-action`, `.destructive-action`, `.linked`, `.osd`, `.card`.
- Toolbars: icon-only / icon+label / menu / split / dropdown buttons are flat automatically; text-only stay raised.
- **Do not shrink below content**: `can-shrink` is false by default; labels in buttons never wrap or clip.

## iced mapping

`widget::*_button`, `style::button::*`. Labels use `Wrapping::None` so the button keeps its natural width.
