# RangeSlider and RangeSliderRow

GTK 4 and libadwaita have no two-thumb scale: `GtkScale` holds one value, and GNOME apps that need an interval (a price filter, a time window) use two scales or two spin buttons. `RangeSlider` is the multi-value sibling of `GtkScale`, in the manner of the Radix and Material range slider: one rail, two thumbs, the accent fill between them.

- The value is a pair `(lo, hi)` with `lo <= hi`. A thumb never passes the other (it stops, or with `Collide::Push` carries it along), and `min_gap` keeps them at least that far apart.
- The rail, fill and handles are those of `GtkScale` (the `style::slider` functions are reused). Hover and drag shade the handle the same way; a focused thumb gets the focus ring; disabled dims rail, fill, handles and marks.
- Pressing on a thumb drags it by its grab point; pressing on the rail moves the nearest thumb there and keeps dragging it. Two thumbs on the same spot: the one that has room to move towards the pointer is grabbed.
- Each thumb is its own focus stop. Arrows step it, `PageUp`/`PageDown` move ten steps, `Home`/`End` go to the ends (stopping against the other thumb).
- `marks` / `marks_every` draw ticks under the rail (no labels).
- `RangeSliderRow` is an `ActionRow` with the slider as suffix and the current `lo – hi` next to it, wrapping below the title like `SliderRow`. `format` writes one end (`$12`, `3 h`).

#### Single and multi-value siblings

| Single value                     | Multiple values                    | Shared rule                                                       |
| -------------------------------- | ---------------------------------- | ----------------------------------------------------------------- |
| `scale` / `SliderRow`            | `RangeSlider` / `RangeSliderRow`   | `primitives::range` (`Track`, `Range`)                            |
| `DateInput`                      | `DateRangeInput`                   | `primitives::range::DateRange`, `primitives::date`                |
| `Dropdown`                       | `MultiSelect` (already multi)      | `primitives::Selection`                                           |

## iced mapping

`widget::RangeSlider`: a custom iced `Widget` (iced's `slider` has one handle), controlled like every component: you pass `(lo, hi)` and turn `on_change((lo, hi))` into your update; the only state in the widget tree is the drag and which thumb has focus. All decisions (nearest thumb, move with the ordering invariant and the gap, snapping, key handling, value to fraction) are `primitives::range::Track`, unit tested, generic over `f32`, `f64`, `i32` and `Date`; the widget is `f32` for now.

Without `on_change`, or with `.disabled(true)`, it ignores input and is not a focus stop. Inside a `Field`, use `.inline(true)`; the slider has no invalid state of its own, so validate with `Track::validity(range)` (for example a minimum width) and give the result to the `Field`.

Deviations: a drag uses the thumb's grab offset (no jump when grabbing a thumb off-centre); no marks labels; no scroll-wheel stepping; thumbs are not reachable by a single `Tab` stop with arrows switching thumbs (each thumb is a stop, as in Radix).
