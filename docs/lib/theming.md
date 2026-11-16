# Theming

Use `iad::Theme` as the application theme. Stock iced widgets are restyled too. Build it from the desktop config unless you have a reason not to: `Theme::from_config(&config::current())`.

Theme files and validation are described in [app theme config](theme-config.md). The [four reference toys](../../theme-toys/README.md#theme-resolution) (`win95`, `winxp`, `win7`, `mate`; `gnome2` compatibility alias only) use independent opt-out bases with optional editor overlays, not the installed desktop theme. The [theme editor](../../theme-editor/README.md) launches the gallery or toys externally; launch appearance is independent of the scheme being edited.

## Layers

```mermaid
flowchart LR
    Config["desktop.conf"] --> Theme
    Palette --> Semantic --> Theme
    Metrics --> Theme
    Typography --> Theme
    Shadows --> Theme
```

| Layer        | Type                    | Holds                                                            |
| ------------ | ----------------------- | ---------------------------------------------------------------- |
| Named colors | `Palette`               | libadwaita variable names (`accent_bg_color`, …), light and dark |
| Semantic     | `Semantic`              | what components ask for: surfaces, status swatches, opacities    |
| Metrics      | `Metrics`, `Typography` | spacing, radii, sizes, text scale                                |

```rust
let theme = iad::Theme::dark()
    .accent(iad::AccentColor::Purple)
    .with_palette(|p| p.window_bg_color = my_bg)   // semantic is re-derived
    .with_semantic(|s| s.card.bg = my_card)        // apply after the palette
    .with_metrics(iad::Metrics::compact());
```

## Rules

- Use roles, not colors. `Role::{Accent, Success, Warning, Error, Destructive}` for status. Never a fixed hue for meaning.
- Text on colored fills is what the theme defines (`accent-fg`, `primary-fg`, `success-fg`, …), as in Bootstrap and Mantine. A theme can opt in to picking light or dark text by contrast with `[colors] auto-contrast=true` (`Theme::with_auto_contrast`). A component's `text_color` or `Theme::with_semantic` overrides either.
- Primary actions (suggested buttons) use the `primary` color, not the accent. The accent is for selection, focus and links, and may follow the desktop; the primary color is the theme's own.
- Use the selected theme's geometry tokens. The libadwaita preset follows HIG radii; validated [theme-file overrides](theme-config.md#tokens) can use different geometry for other design languages.
- Shadows are settings: `Shadows::none()`, `.with_scale`, `.with_intensity`. A card draws each layer as its own rounded shape, so the shadow follows the border.
- Density: `Metrics::compact()` or `[appearance] density=compact`.
- Dimmed text is for secondary information and the disabled state. Not for chosen values.

## Accent

One of nine names, or any color: `AccentColor::Custom(0xRRGGBB)`, `AccentColor::custom(r, g, b)`, `AccentColor::parse("#3584e4")`. `[appearance] accent` takes a name or `#rrggbb`. A custom accent derives its standalone text color for light and dark.

`accent_picker(selected, on_select).custom(text, on_text)` adds a swatch that opens a `ColorInput`. `.dialog(true)` opens it as a dialog.

## Transparency

```rust
Theme::light().with_transparency(0.4)    // [appearance] transparency=0.4
```

- Every surface background keeps its color and has its alpha scaled by `1 - t`: window, view, bars, sidebars, cards.
- Popovers and dialogs scale by `1 - t/2`, so their text stays legible.
- Foregrounds, accents and hairlines are not changed.
- Tints derived from a surface (hover, press, `color::lighten`, `darken`, `tint`) keep its alpha. A translucent surface stays translucent in every state.
- The window must be transparent. Client-side windows are. `windows::settings()` makes a server-side one transparent when `transparency` is above 0.

## Tint

```rust
Theme::light().with_tint(Some(Color::from_rgba(1.0, 0.0, 0.0, 0.3)))   // [appearance] tint=#ff00004d
```

Off by default. A color of its own, not the accent. Surfaces move towards it by its alpha. Their own alpha is kept, so tint and transparency combine in either order.

`[appearance] tint` takes `none`, `#rrggbb` (strength 25%) or `#rrggbbaa`.

## Layout tokens are fixed at build time

iced resolves colors at draw time and layout at view-build time. Padding and sizes come from `Metrics::desktop()` unless you pass `.metrics(m)` to a component. Colors, radii and opacities follow the theme.
