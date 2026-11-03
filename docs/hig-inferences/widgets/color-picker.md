# ColorPicker and ColorInput

No libadwaita widget (`GtkColorDialog` opens a system dialog). After Mantine's [ColorPicker](https://mantine.dev/core/color-picker/) and [ColorInput](https://mantine.dev/core/color-input/).

## What it is

`ColorPicker` is a saturation / value square, a hue slider and an optional alpha slider, with swatches and a readout. `ColorInput` is the form input built on it: the color in a box (like `GtkColorDialogButton`) that opens the picker with an entry for the color code, in a [Popover](popover.md) or, with `dialog(true)`, a dialog over a faded, disabled window (`scrim(false)` keeps it clear; `close_on_outside(false)` keeps a press outside from closing it).

## Anatomy and behavior

- **Color** ([`primitives::color_picker`](../../lib/primitives.md)). `Hsva` (hue in degrees, saturation, value, alpha): what the controls edit, so a gray or a black keeps its hue while you drag across it. Conversions to and from 8-bit RGB and HSL.
- **Text forms.** `format(ColorFormat::..)`: hex, hex with alpha, rgb, rgba, hsl, hsla, hsv, hsva. `Hsva::parse` reads any of them and the short hex forms, with or without the `#`.
- **Controls.** Press or drag the square for saturation and value, the sliders for hue and alpha; a drag keeps working outside the control.
- **Swatches.** `swatches([..])` are one-press choices; ones `parse` cannot read are left out.
- **ColorInput.** The app keeps the text and the color: `on_input` gets what was typed (parse it with `Hsva::parse`), `on_change` a color picked with the controls (format it with `Hsva::format`). Text that does not parse shows the error state. `trigger(..)` replaces the box.
- **AccentPicker.** `accent_picker(selected, on_select).custom(text, on_text)` adds a last swatch that opens a `ColorInput` with the nine accents as swatches; the choice is an `AccentColor::Custom`. It takes `dialog` and `scrim` too.
- Controlled: the app owns the color.

## iced mapping

`widget::ColorPicker` draws its three controls in one widget (the square and the hue slider with iced gradients, a checkerboard under the alpha slider). Not included: nudging with the keyboard (iced has no focus model), and an eyedropper.
