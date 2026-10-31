# Metrics

The docs give very few numbers. Everything marked **‡** is from memory of the
stylesheet source and should be checked against the SCSS.

| Item                                 | Value                                                                               | Source               |
| ------------------------------------ | ----------------------------------------------------------------------------------- | -------------------- |
| Toolbar spacing and padding          | 6px (`.toolbar`); OSD toolbar 12px                                                  | ✓ `_toolbars.scss`   |
| ActionBar padding                    | 7 6 6 6, spacing 6                                                                  | ✓                    |
| SearchBar padding                    | 6 6 7 6                                                                             | ✓                    |
| Frame border                         | 1px solid border color (15%)                                                        | docs                 |
| `$button_radius`                     | **9px**                                                                             | ✓ `_common.scss`     |
| `$card_radius`                       | 12px                                                                                | ✓                    |
| `$menu_radius`                       | 9px; `$menu_margin` 6, `$menu_padding` 12                                           | ✓                    |
| `$popover_radius`                    | menu radius + 6 = **15px**                                                          | ✓                    |
| `$dialog_radius` / `--window-radius` | button radius + 6 = **15px**                                                        | ✓                    |
| Default window minimum size          | 360×200                                                                             | docs (`AdwWindow`)   |
| Button                               | min-height 24 + padding 5×10 = **34**; min-width 16                                 | ✓ `_buttons.scss`    |
| Raised button fill                   | 10% rest, 15% hover, **30% active**; checked 30/35/40%                              | ✓                    |
| Suggested hover / active             | overlay `currentColor` 10% / `rgb(0 0 6 / 20%)`                                     | ✓                    |
| Circular button                      | 34×34, radius 9999                                                                  | ✓                    |
| Pill button                          | padding 10×32, radius 9999                                                          | ✓                    |
| OSD button                           | min 32×32, `rgb(0 0 0 / 65%)`                                                       | ✓                    |
| Entry min height                     | 34                                                                                  | ‡                    |
| Row header min height                | **50** (action, `has-body`, entry); button row 40                                   | ✓ `_lists.scss`      |
| `.rich-list` row                     | padding 8×12, min-height 32                                                         | ✓                    |
| Activatable row hover / active       | 3% / 8% foreground                                                                  | ✓                    |
| Boxed-list row separator             | 1px `--card-shade-color`                                                            | ✓                    |
| `boxed-list-separate` gap            | 12px                                                                                | ✓                    |
| Popover `contents` padding           | 8px                                                                                 | ✓ `_popovers.scss`   |
| Scrollbar slider                     | 8px (min length 40); idle overlay 3px, hover 8px; trough 10% on hover; 200ms linear | ✓ `_scrolling.scss`  |
| Switch                               | ≈46×26 (knob 20)                                                                    | ‡                    |
| Header bar                           | min-height **47**, padding `6 7 7 7`, `inset 0 -1px --headerbar-shade-color`        | ✓ `_header-bar.scss` |
| Flat header bar (`%headerbar-flat`)  | `background: none`, `box-shadow: none`, `color: inherit`, min-height **46**         | ✓                    |
| Navigation sidebar row               | height ≈36, 6px list padding, 2px gap                                               | ‡                    |
| Progress bar thickness               | 8                                                                                   | ‡                    |
| Scale trough / handle                | 10 / 20                                                                             | ‡                    |
| Split view sidebar                   | fraction 25%, min 180sp, max 280sp                                                  | docs                 |
| Clamp example margins                | 24 vertical, 12 horizontal, 24 spacing                                              | docs (example)       |
| WrapBox example spacing              | 6 / 6                                                                               | docs (example)       |

See also [shadows.md](shadows.md).

## Radii are capped

Per project decision the HIG/Adwaita radii are the **maximum**: small 4,
control 9 (`$button_radius`), menu 9, card 12, popover 15, window/dialog 15
(`--window-radius`), pill 999. Themes may shrink radii but never exceed these
(`Metrics::clamped`). _Correction:_ an earlier revision used 6px for controls
from memory; the stylesheet says 9.

## `sp`

Breakpoints use `sp`: pixels scaled by the system text scale. iced has no text
scale factor, so `sp` = logical px here.
