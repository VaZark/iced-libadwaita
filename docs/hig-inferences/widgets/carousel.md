# Carousel

libadwaita: [`Carousel`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Carousel.html)

## What it is

Paginated swipeable pages (orientation, spacing, reveal duration).

## Anatomy and behavior

`allow-long-swipes`, `allow-mouse-drag`, `allow-scroll-wheel`, `interactive`, `position`, `spacing` (px), `reveal-duration` (ms).

## CSS nodes and classes

`carousel`.

## iced mapping

`carousel(pages, index, on_navigate).indicator(Dots | Lines | None).spacing(px).interactive(bool)`. The position scrolls on libadwaita's default scroll spring (damping ratio 1, mass 0.5, stiffness 500) and the indicator follows it (dots grow and brighten gradually; the active line glides). Mouse-drag swipes with rubber-banding at the ends; on release the drag velocity becomes the spring's initial velocity and the carousel settles on the nearest page. Clicking an indicator navigates. State is controlled (`index`).

Pages remain caller-owned elements. The carousel clips moving pages to its viewport and carries the active `layouts::boundary` ancestor intersection into translated page coordinates for drawing, pointer events and hit tests. It does not invent a card radius. `Card` supplies a boundary from its final surface style, so its content clip, border and elevation share geometry even with custom asymmetric corners; nested surfaces intersect rather than replace ancestor constraints.

Boundary geometry follows the actual iced subtree in a scoped stack, restored between siblings. Child overlays are explicitly forwarded as portals outside the card's boundary and own their own surfaces. Rounded clipping is a conservative inward 1-logical-pixel staircase using repeated draws in rectangular layers, not a native antialiased mask; large radii increase draw cost. See [motion and boundaries](../../lib/motion.md).

## Deviations / gaps

Mouse drag only (no touch/scroll-wheel/keyboard yet); no `reveal-duration` for inserting/removing pages; no long-swipe setting.
