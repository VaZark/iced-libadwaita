# ButtonContent

libadwaita: [`ButtonContent`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ButtonContent.html)

## What it is

Icon + label helper for buttons; sets `.image-text-button` and wires mnemonics. `can-shrink`.

## CSS nodes and classes

`buttoncontent` > `box` > `image`, `label`.

## iced mapping

`ButtonContent::new(label).icon(icon)`. Label does not wrap.
