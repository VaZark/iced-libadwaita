# FloatingIndicator

No libadwaita counterpart. After Mantine's [FloatingIndicator](https://mantine.dev/core/floating-indicator/), the "custom variants" of its tabs.

## What it is

A row of the app's own widgets (usually flat buttons) with a highlight that **slides behind the active one**. It is how the gallery's Preview / Code tabs are drawn.

## Anatomy and behavior

- **Controlled.** The app owns the active index and the children's messages; the widget only places the children and draws the highlight.
- **Highlight.** Supply `target_shape` to share the target's effective corners at draw time, using the current theme, active index and animated bounds. Arbitrary children are unhighlighted without known geometry; `.radius(px)` is an explicit uniform-corner override, not automatic style discovery. The default color is the window foreground at 10% (`color` overrides it).
- **Motion.** The full target rectangle (position, width and height) eases with `selection` motion (200 ms ease-out-quad, instant under reduced motion), not the row's maximum height. First placement is immediate. Four corners come from the resolver for the current target rather than a second theme-radius lookup.
- Children keep their own pointer behavior and overlays.

## iced mapping

`widgets::FloatingIndicator`, generic over the renderer. The resolver describes the existing target; it does not apply a style to arbitrary child elements. `Tabs` supplies its own shared button-style resolver and title bounds, excluding a sibling close button. Ancestor surface boundaries constrain the subtree but do not substitute for target-specific geometry. Not included: a vertical orientation, and keyboard arrow navigation between the children (iced has no focus model for it yet).
