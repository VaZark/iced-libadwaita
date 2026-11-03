# ColorPicker and ColorInput

No libadwaita widget (`GtkColorDialog` opens a system dialog). After Mantine's [ColorPicker](https://mantine.dev/core/color-picker/) and [ColorInput](https://mantine.dev/core/color-input/).

## What it is

- `ColorPicker`: a saturation/value square, a hue slider, an optional alpha slider, swatches and a readout.
- `ColorInput`: the form input. The color in a box (like `GtkColorDialogButton`) that opens a `ColorPicker` with an entry for the color code. The picker is a pure picker. The input builds on it.

## Parameters

| Component     | Param               | Meaning                                         | Default    |
| ------------- | ------------------- | ----------------------------------------------- | ---------- |
| `ColorPicker` | `alpha`             | alpha slider                                    | off        |
|               | `swatches([..])`    | one-press colors (hex or any form `parse` reads) | none      |
|               | `format`, `readout` | readout form, show it                           | hex, on    |
|               | `width`             |                                                 | 240        |
| `ColorInput`  | `alpha`, `swatches` | passed to the picker                            |            |
|               | `trigger(..)`       | replaces the color box                          | box        |
|               | `dialog(bool)`      | open as a dialog                                | popover    |
|               | `scrim`             | fade the window (dialog)                        | on         |
|               | `close_on_outside`  | a press outside closes the dialog               | on         |
|               | `window_radius`, `disabled`, `open_initially` |                              |            |

## Behavior

- **Color.** `Hsva` (hue in degrees, S, V, A) is what the controls edit, so a gray keeps its hue while you drag across it. Conversions to 8-bit RGB and HSL are lossless.
- **Text forms.** hex, hexa, rgb, rgba, hsl, hsla, hsv, hsva. `Hsva::parse` reads all of them and the short hex forms, with or without `#`.
- **Controls.** Press or drag the square (S, V) and the sliders (hue, alpha). A drag keeps working outside the control.
- **ColorInput.** The app keeps the text and the color. `on_input` gets typed text (parse it with `Hsva::parse`). `on_change` gets a picked color (format it with `Hsva::format`). Text that does not parse is the error state. The code entry takes focus when it opens (`initial_focus`).
- **AccentPicker.** `accent_picker(selected, on_select).custom(text, on_text)` adds a last swatch that opens a `ColorInput` with the nine accents as swatches. The choice is an `AccentColor::Custom`. It takes `dialog`, `scrim`, `close_on_outside`.

## iced mapping

`ColorPicker` draws its controls in one widget: iced gradients for the square and the hue slider, a checkerboard under the alpha slider. `ColorInput` is a `Popover` (or `Presentation::Dialog`) around a `ColorPicker` and an `Entry`.

## Deviations

No keyboard nudging (no focus model for the controls). No eyedropper.
