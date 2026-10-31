# Shadows

Source: libadwaita stylesheet (`_lists.scss`, `_popovers.scss`, `_dialogs.scss`,
`_toolbars.scss`, `_scrolling.scss`, `_common.scss`). The published docs only name
the shade colors; the geometry below is quoted from the SCSS. Light-style values;
the sheet uses the same layer geometry in dark and relies on translucent fills and
darker shade colors, so dark variants are otherwise unverified (‡).

## Principles

* Elevation is a stack of CSS box-shadows: usually a **1px ring** (`0 0 0 1px`) plus
  one or two **soft blurred layers**. Never a single hard drop shadow.
* Colors are near-black `rgb(0 0 6 / x)` ("ink") at very low alpha (3–10%). Shade
  CSS variables (`--shade-color`, `--card-shade-color`, `--headerbar-shade-color`,
  `--popover-shade-color`) change between light and dark; the ring/blur alphas do not.
* **Flat by default.** Bars (header, tab, action) are flat; separation comes from the
  window/headerbar color difference plus a scroll *undershoot*, not a standing shadow.
  `raised` / `raised-border` are opt-in styles.
* High contrast replaces the ring with a `--border-color` outline.
* Transitions for shadow/background: 200ms ease-out (`$ease-out-quad`).

## Layers

| Surface | box-shadow | Source |
|---|---|---|
| Card / boxed list | `0 0 0 1px ink 3%`, `0 1px 3px 1px ink 7%`, `0 2px 6px 2px ink 3%` | `_lists.scss` (`boxed-list-separate` row) |
| Popover / menu | `0 0 0 1px black 5%`, `0 1px 5px 1px black 9%`, `0 2px 14px 3px black 5%` | `_popovers.scss` |
| Floating dialog | `0 0 14px 2px ink 3%`, `0 0 5px 2px ink 10%`, `0 0 0 1px black 5%` | `_dialogs.scss` (`floating-sheet > sheet`) |
| OSD popover | `0 0 1px white 10%` | `_popovers.scss` |
| Raised top bar | `0 1px headerbar-shade@50%`, `0 2px 4px headerbar-shade@50%` | `_toolbars.scss` |
| Raised bottom bar | `0 -1px …`, `0 -2px 4px …` (mirrored) | `_toolbars.scss` |
| Raised-border bar | `0 ±1px --headerbar-darker-shade-color` | `_toolbars.scss` |
| Action bar (raised) | `inset 0 1px --headerbar-shade-color` | `_toolbars.scss` |
| Search bar (raised) | `inset 0 -1px --headerbar-shade-color` | `_toolbars.scss` |
| Drop target row | adds `inset 0 0 0 1px $drop_target_color` | `_lists.scss` |
| Windows (CSD) | not in the fetched sources ‡ | |

## Dim layer ("scrim")

`floating-sheet > dimming`: `--shade-color` with **alpha doubled** (light 7%→14%,
dark 25%→50%).

## Scrollbar edge

Overlay slider outline: `1px solid --scrollbar-outline-color` at 35% (rest) / 60%
(hover); see [../gtk4/scrollbar.md](../gtk4/scrollbar.md).

## iced mapping

`Shadows` (on `Theme`) holds one `Elevation` (the layer list above) per surface
plus three settings:

* `enabled`: master switch (`Shadows::none()` for a flat look)
* `scale`: multiplies offsets and blur radii
* `intensity`: multiplies layer alpha

```rust
let theme = Theme::dark()
    .with_shadows(Shadows::default().with_scale(1.5).with_intensity(0.6));
```

iced draws **one** shadow per quad, so `Shadows::resolve` splits an elevation into
the 1px **ring** (drawn as the container border) and the blurred layers **merged**
into one iced `Shadow` (alpha-weighted offset, largest blur+spread, summed alpha
capped at 0.5). It is an approximation of stacked shadows.
Raised bars are drawn as a 6px gradient strip because later siblings would paint
over a real shadow. No `inset` layers, no window shadow, no transitions.

## Drawing a shadow that follows the border

iced draws one shadow per quad and has no *spread*, so merging Adwaita's stacked layers into a
single blurred shadow gives a rectangular halo that ignores the card radius. Cards use the CSS
trick instead: each soft layer is its own shape behind the surface, **inflated by the layer's
spread, with its corner radius grown by the same amount**, offset by the layer's `x y` and blurred
by its blur. The 1px ring stays the border. `widget::card_shadow` does this for `Card` (and so for
`boxed_list` and preference groups); a plain `container(..).style(style::container::card)` still
gets the single merged shadow.
