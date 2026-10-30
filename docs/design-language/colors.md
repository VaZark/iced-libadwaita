# Colors

Source: [CSS variables](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html), [styles and appearance](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/styles-and-appearance.html).
Values are **light / dark**. `*` = outside sRGB gamut, clipped.

## Accent (system preference)

| Accent | bg (`--accent-bg-color`) | fg | standalone light | standalone dark |
|---|---|---|---|---|
| Blue (default) | #3584e4 | #fff | #0461be | #81d0ff* |
| Teal | #2190a4 | #fff | #007184* | #7bdff4 |
| Green | #3a944a | #fff | #15772e | #8de698 |
| Yellow | #c88800 | #fff | #905300* | #ffc057* |
| Orange | #ed5b00 | #fff | #b62200* | #ff9c5b* |
| Red | #e62d42 | #fff | #c00023* | #ff888c* |
| Pink | #d56199 | #fff | #a2326c | #ffa0d8* |
| Purple | #9141ac | #fff | #8939a4 | #fba7ff* |
| Slate | #6f8396 | #fff | #526678 | #bbd1e5 |

Per-widget override must set **both** `--accent-bg-color` and `--accent-color`.

## Semantic groups

| Group | bg | fg | standalone (`--*-color`) |
|---|---|---|---|
| destructive | #e01b24 / #c01c28 | #fff | #c30000* / #ff938c* |
| success | #2ec27e / #26a269 | #fff | #007c3d* / #78e9ab |
| warning | #e5a50a / #cd9309 | rgb(0 0 0 / 80%) | #905400* / #ffc252 |
| error | #e01b24 / #c01c28 | #fff | #c30000* / #ff938c* |

`.success` / `.warning` / `.error` also retint the accent (entries use them for
validation). Success/warning are the high/low offsets of `GtkLevelBar`.

## Surfaces

| Surface | bg | fg |
|---|---|---|
| window | #fafafb / #222226 | rgb(0 0 6 / 80%) / #fff |
| view | #fff / #1d1d20 | same fg |
| headerbar | #fff / #2e2e32 | same fg |
| card | #fff / rgb(255 255 255 / 8%) | same fg |
| dialog | #fafafb / #36363a | same fg |
| popover | #fff / #36363a | same fg |
| sidebar | #ebebed / #2e2e32 | same fg |
| secondary sidebar | #f3f3f5 / #28282c | same fg |
| overview (tab overview) | #f3f3f5 / #28282c | same fg |
| thumbnail | #fff / #39393d | same fg |
| active toggle | #fff / rgb(255 255 255 / 20%) | same fg |

Supporting colors (always translucent black, "rgb(0 0 6 / x)"):

| Variable | light | dark |
|---|---|---|
| `--card-shade-color` (row separators) | 7% | 36% |
| `--popover-shade-color` | 7% | 25% |
| `--headerbar-shade-color` | 12% | 36% |
| `--headerbar-darker-shade-color` | 12% | 90% |
| `--headerbar-border-color` | = fg | = fg (vertical borders only) |
| `--headerbar-backdrop-color` | = window bg | = window bg (unfocused) |
| `--sidebar-border-color` | 7% | 36% |
| `--sidebar-shade-color` | 7% | 25% |
| `--sidebar-backdrop-color` | #f2f2f4 / #28282c | unfocused |
| `--secondary-sidebar-border/shade` | 7% / 7% | 36% / 25% |
| `--shade-color` | 7% | 25% |
| `--scrollbar-outline-color` | #fff | rgb(0 0 6 / 50%) |

## Helper opacities (do not override)

| Variable | regular | high contrast | used for |
|---|---|---|---|
| `--border-opacity` | 15% | 50% | borders via `--border-color` = `currentColor` at that alpha |
| `--dim-opacity` | 55% | 90% | `.dimmed`, subtitles |
| `--disabled-opacity` | 50% | 40% | insensitive widgets |

`--window-radius` = **15px** (follows window state; used to round focus rings
near window edges).

## Standalone derivation
`color: oklab(from <bg> var(--standalone-color-oklab))` where the variable is
`min(l, 0.5) a b` (light) / `max(l, 0.85) a b` (dark).

## Backdrop
Unfocused windows swap in `--headerbar-backdrop-color`,
`--sidebar-backdrop-color`, … so bars match the window background.

## Palette (GNOME)
`--blue-1..5`, `--green`, `--yellow`, `--orange`, `--red`, `--purple`,
`--brown`, `--light`, `--dark` (five shades each, e.g. blue-3 `#3584e4`).

## iced mapping
`Palette` holds these names; `Semantic` is the role layer; opacities live in
`Semantic::opacity`. Palette values track the table above.
