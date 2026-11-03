# ColorPicker and ColorInput

No libadwaita widget (`GtkColorDialog` opens a system dialog). After Mantine's [ColorPicker](https://mantine.dev/core/color-picker/) and [ColorInput](https://mantine.dev/core/color-input/).

## What it is

`ColorPicker` is a saturation / value square, a hue slider and an optional alpha slider, with swatches and a readout. `ColorInput` is an entry for the color with the picker in a [Popover](popover.md) behind a swatch.

## Anatomy and behavior

- **Color** ([`primitives::color_picker`](../../lib/primitives.md)). `Hsva` (hue in degrees, saturation, value, alpha): what the controls edit, so a gray or a black keeps its hue while you drag across it. Conversions to and from 8-bit RGB (lossless for every 8-bit color) and HSL.
- **Text forms.** `format(ColorFormat::..)`: hex, hex with alpha, rgb, rgba, hsl, hsla, hsv, hsva. `Hsva::parse` reads any of them and the short hex forms (`#f00`, `#f008`), with or without the `#`.
- **Controls.** Press or drag the square for saturation and value, the sliders for hue and alpha; a drag keeps working outside the control.
- **Swatches.** `swatches([..])` (any text `parse` reads; the others are left out) are one-press choices.
- **ColorInput.** The app keeps the text (so typing is not interrupted) and the color: `on_input` gets what was typed (parse it with `Hsva::parse`), `on_change` a color picked in the popover (format it with `Hsva::format`). Text that does not parse shows the error state.
- Controlled: the app owns the color.

## iced mapping

`widget::ColorPicker` draws its three controls in one widget (the square and the hue slider with iced gradients, a checkerboard under the alpha slider). Not included: nudging with the keyboard (iced has no focus model), and an eyedropper.
