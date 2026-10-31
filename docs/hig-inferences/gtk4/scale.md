# Scale

GTK4: [`GtkScale`](https://docs.gtk.org/gtk4/class.Scale.html)

## What it is

Slider for picking a numeric value (extends `GtkRange`); marks, optional value text, origin/highlight, fine-tune with Shift. Equal bounds hide the slider.

## CSS nodes

```
scale[.fine-tune][.marks-before][.marks-after]
├── value[.top|.right|.bottom|.left]
├── marks.top
│   ╰── mark (indicator, [label])
├── marks.bottom
╰── trough
    ├── fill
    ├── highlight
    ╰── slider
```

## Style classes and states

`.fine-tune`, `.marks-before/after`, positional classes on `marks` and `value`.

## Spec

Trough ≈10px, slider ≈20px circle ‡.

## Adwaita defers

Accent highlight over a translucent trough; white round slider with a subtle border.

## iced mapping

`style::slider::default` (stock iced `slider`). No marks/value text.
