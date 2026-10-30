# Scrollbar

GTK4: [`GtkScrollbar`](https://docs.gtk.org/gtk4/class.Scrollbar.html)

## What it is

A horizontal or vertical scrollbar driven by a `GtkAdjustment` (`value` within `lower … upper − page-size`; `step-increment`, `page-increment`). Role: scrollbar.

## Behavior

* **Overlay scrolling** (`GtkScrolledWindow:overlay-scrolling`, default on when no mouse is detected): narrow auto-hiding indicators float over the content instead of taking layout space. Disabling it gives classic always-visible scrollbars.
* The overlay indicator is thin and faint at rest, and **grows and gains a trough when hovered or dragged**.
* Dragging moves the slider; clicking the trough pages; fine-tune (Shift/ctrl drag) slows movement.

## CSS nodes

```
scrollbar.horizontal|.vertical
╰── range[.fine-tune]
    ╰── trough
        ╰── slider
```

## Style classes and states

Inside a `GtkScrolledWindow` the scrollbar also gets:
* position: `.left` `.right` `.top` `.bottom`
* overlay scrolling: `.overlay-indicator`, `.hovering`, `.dragging`
* `.fine-tune` on `range` while fine-tuning (slow drag)

The GTK page gives no sizes and does not describe when the overlay classes apply; the behavior below is from the Adwaita theme.

## Spec

Verified against libadwaita `_scrolling.scss` (Adwaita wins over GTK's own theme):

| Item | Value |
|---|---|
| Slider width | `$_slider_width` **8px** (min-width/min-height), hit area = 8px transparent border (margin −8px) |
| Slider min length | **40px** (`$_slider_min_length`) |
| Slider shape | radius 99px |
| Slider fill | `currentColor` **20%** rest, **40%** `:hover`, **60%** `:active`; high-contrast 40/60/100% |
| Trough | radius 99px; margin 9px (8px on the cross axis) |
| Overlay indicator, idle | slider **3px**; trough margins 4px; slider outline 35% of `--scrollbar-outline-color` (60% in high contrast) |
| Overlay indicator, `.hovering` | slider grows to **8px**; trough fill `currentColor` **10%** (25% high contrast) |
| Slider outline | `1px solid` `--scrollbar-outline-color` at 60% (rest, hovering), outline-offset −8px |
| `.fine-tune` | slider `--accent-color` at 60% |
| Disabled | slider opacity 0 |
| Transition | all 200ms linear |
| `--scrollbar-outline-color` | light `#fff`, dark `rgb(0 0 6 / 50%)` (libadwaita docs) |

Undershoot/overshoot values are produced by mixins not included in the fetched file (unknown).

## Adwaita defers

* libadwaita adds `--scrollbar-outline-color` and derives the slider from `currentColor`; use those rather than GTK's `@theme_*` scrollbar colors.
* `AdwToolbarView` replaces the `.undershoot-*` scroll shadows, so we do not draw GTK undershoot/overshoot.

## iced mapping

`adw::widget::scrolled_window(content)` configures iced's `scrollable` with Adwaita proportions instead of iced's default 10px bar: a floating (overlay, non-layout) scrollbar: 8px rail with a 4px slider, 2px from the edge (Adwaita: 3px idle growing to 8px on hover; iced cannot resize the slider on hover, so 4px is a fixed compromise). `style::scrollable::default` follows the SCSS: slider `currentColor` at 20% rest / 40% hovered / 60% dragged, a 10% trough only while hovered, a 1px `--scrollbar-outline-color` outline at 35% (rest) / 60% (hover), fully rounded. All values live in `Metrics` (`scrollbar_width`, `scrollbar_slider_width`, `scrollbar_margin`).
**Gaps:** iced cannot animate size, so the slider does not *grow* on hover (we widen the rail visually through the trough); no auto-hide fade or 200ms transition; no 40px minimum slider length; no fine-tune accent.
