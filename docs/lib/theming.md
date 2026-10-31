# Theming

Use `adw::Theme` as the application theme; stock iced widgets are restyled too. Build it from the
desktop config unless you have a reason not to: `Theme::from_config(&config::current())`.

#### Layers

| Layer        | Type                    | Holds                                                                  |
| ------------ | ----------------------- | ---------------------------------------------------------------------- |
| Named colors | `Palette`               | libadwaita-style variable names (`accent_bg_color`, …), light and dark |
| Semantic     | `Semantic`              | What components ask for: surfaces, status swatches, opacities          |
| Metrics      | `Metrics`, `Typography` | Spacing, radii, sizes, text scale                                      |

```rust
let theme = adw::Theme::dark()
    .accent(adw::AccentColor::Purple)
    .with_palette(|p| p.window_bg_color = my_bg)   // semantic re-derived
    .with_semantic(|s| s.card.bg = my_card)        // apply after the palette
    .with_metrics(adw::Metrics::compact());
```

#### Rules of thumb

- **Use roles, not colors.** `Role::{Accent, Success, Warning, Error, Destructive}` for status;
  never a fixed hue for meaning.
- **Text on colored fills is chosen by contrast** (CSS `color-contrast()`): light or dark,
  whichever contrasts more. Override with a component's `text_color`, `Theme::with_semantic`, or
  turn it off with `with_auto_contrast(false)`.
- **Radii are capped** at the HIG values (controls 9, cards 12, popovers and windows 15); you can
  lower them (`radius_m`, `radius_l`, `radius_toast`), not raise them.
- **Shadows are settings**: `Shadows::none()`, `.with_scale`, `.with_intensity`. Cards draw each
  layer as its own rounded shape so the shadow follows the border.
- **Density**: `Metrics::compact()` or `[appearance] density=compact`.
- Dimmed text is for secondary information and the disabled state, not for chosen values.

#### Limitation: layout tokens are fixed at build time

iced resolves colors at draw time but layout at view-build time, so padding and sizes come from
`Metrics::desktop()` unless you pass `.metrics(m)` to a component. Colors, radii and opacities
follow the theme automatically.
