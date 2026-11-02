# Resizable

A frame with a drag handle, like a textarea's grip, for trying **responsive components at any size without resizing the window**: a `MetaComponent`, a `ResponsiveTable`, a `SidebarList::cards_below`, a docked layout. Nothing in libadwaita or Mantine does this in a component; it is closest to the browser's responsive design mode and to the resizable preview frame in component playgrounds (Storybook viewports, Mantine's docs).

- **Controlled.** The app owns the frame `Size` and takes it from `on_resize`; the content is laid out at exactly that size, so container-based decisions (a `MetaComponent` trigger, a table's fit) see the frame, not the window. The frame is clamped to the room its parent gives it, and `on_resize` then reports the size it actually got (once, on the next redraw), so the app's `Size` never claims more than is shown.
- **Handle.** The bottom-right corner by default (dots, a pointer that turns into a resize cursor, brighter while dragged); `grip(Grip::Right | Bottom | Left | Top | any corner)` for an edge or another corner. An edge changes one axis only; the left and top edges grow against the pointer.
- **Constraints.** `Constraints::new(min, max)`, a grid `step(px)`, and `snap_to(points, radius)`: the **width snaps to the breakpoints** when within the radius, so the edge of each width class is easy to hit and to stay on.
- While the handle is held the content does not see the pointer; `border(false)` removes the hairline.

## iced mapping

`primitives::resize` is pure and unit tested (`drag`, `step`, grips, min / max, grid and snapping); `widget::Resizable` is generic over the renderer so its layout is tested without a window. Overlays of the content are forwarded.

The gallery's Layout > Resizable page puts a `MetaComponent`, a `ResponsiveTable` and the **embedded AppShell** (no window chrome of its own; the frame's width is fed to its `Adaptive` as `Message::Resized`) in frames, so the mobile / tablet / desktop layouts and their transitions can be tried by dragging.

Not included: keyboard resizing of the focused handle (the primitive has `step` for it, but iced has no focus model for it yet), touch handles larger than the hit area, and a built-in size readout (the gallery draws one).
