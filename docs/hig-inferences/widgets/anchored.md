# Indicator and docked surfaces

`Indicator` and the `dock_*` helpers are the widgets of the anchor primitive (`primitives::anchor`) for _inline_ layout: a base widget keeps its own size, and a second surface is placed against its bounds, over it.

- **Indicator** is Mantine's [`Indicator`](https://mantine.dev/core/indicator/): a badge, dot or count centered on a corner of an avatar, button or icon (`Relation::Straddle`). `corner(side, align)` picks the corner (default top right), `offset` moves it in from the corner on both axes, `cross_offset` slides it along the edge, `show(false)` removes it (Mantine `disabled`). The badge's own size is accounted for, so a round dot and a "99+" pill both sit exactly on the corner.
- **Docked** surfaces use `Relation::Inside`: `Anchored` with `Placement::inside(side)` plus `.sizing(..)` is the general form. The surface is drawn over the base, in its own layer, and gets events first.
- Placement and sizing are the pure `place_sized` and `Sizing::limits` (unit tested); the widget lays the surface out **once**, with the limits those rules give, because iced text widgets lose their text when laid out twice against the same tree.

## iced mapping

`widget::Anchored` is generic over the renderer (so its layout is unit tested without a window). Popovers of both children are forwarded as overlays. There is no viewport inside a layout, so an `Outside` placement does not flip or shift here: use `Popover` for popovers (it works inside the region it is kept in).

Deviations from Mantine: no `processing` pulse, no `inline`/`withBorder` (draw the ring in the badge's own style), `size` belongs to the badge.
