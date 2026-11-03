# FloatingIndicator

No libadwaita counterpart. After Mantine's [FloatingIndicator](https://mantine.dev/core/floating-indicator/), the "custom variants" of its tabs.

## What it is

A row of the app's own widgets (usually flat buttons) with a highlight that **slides behind the active one**. It is how the gallery's Preview / Code tabs are drawn.

## Anatomy and behavior

- **Controlled.** The app owns the active index and the children's messages; the widget only places the children and draws the highlight.
- **Highlight.** The window foreground at 10% by default (`color`, `radius`); it eases to the active child's place with the `selection` motion (200 ms ease-out-quad, instant under reduced motion). The first time it is drawn it is already in place, so it does not slide in from the corner.
- Children keep their own pointer behavior and overlays.

## iced mapping

`widget::FloatingIndicator`, generic over the renderer so its layout is tested without a window. Not included: a vertical orientation, and keyboard arrow navigation between the children (iced has no focus model for it yet).
