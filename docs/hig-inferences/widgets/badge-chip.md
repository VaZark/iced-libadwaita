# Badge and Chip

libadwaita has no badge or chip widget; GNOME apps build them from the `.tag`-like pattern (a small pill with a tinted background) and, for counts, `AdwViewSwitcherBadge`-style numbers. Our `tag()` helper is the original; these two are the generic versions.

- **Badge**: read-only label or count. Tinted by a `Role` (translucent background, role color text) or `solid(true)` for notification counts. `Badge::count(n, max)` renders `99+`.
- **Chip**: a button-like pill with an optional icon, `on_press`, a trailing ✕ (`on_remove`), a `selected` (solid) state and a disabled state (dimmed, no events).

## iced mapping

`widget::Badge` (container styled by `style::container::tinted` / `role`) and `widget::Chip` (a pill container holding transparent buttons: the body, and the ✕ as its own hit target inside the same pill). Colors come from the theme's semantic swatches, so they follow the scheme and accent. Text is 13px (`space_m + 1`), the size of `tag()`.
