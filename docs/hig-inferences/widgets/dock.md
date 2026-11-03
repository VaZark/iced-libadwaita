# Dock

libadwaita: [`BottomSheet`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.BottomSheet.html) is the bottom case of it; a footer, a banner and a drawer are the others.

## What it is

A surface docked against an edge of a base. One widget for what GNOME and Mantine call a bottom sheet, a drawer, a footer bar and a banner. It belongs with the dialogs: a modal dock is a dialog attached to an edge.

## Anatomy and behavior

- **Side.** `side(Side)` is the edge (default the bottom).
- **Size and length.** You define them: `size(DockSize)` is how far it reaches across the edge and `length(DockSize)` how long it is along it (default the whole edge); each is `Auto` (its own), `Fixed(px)`, a `Fraction` of the base or `FullScreen`. `align(Align)` places a shorter dock along the edge: the start, the center or the end. `FullScreen` as the size covers the base, every border flush (margin ignored, no border, square corners or the window's radius).
- **Margin.** `margin(px)` stands the surface off the edge it is docked to, and off that edge only (not on the other sides). `0` (the default) is **flush** with the edge; a margin lifts it and rounds its corners on that edge.
- **Border.** A hairline border (the gallery previews' kind, `border(false)` removes it) outlines the surface on the sides that are **not** flush with the base: the docked edge (once it stands off), the opposite side (when `size` does not reach it) and the ends along the edge (when it is shorter and not aligned to them). Flush sides have none.
- **Mode.** `Push` (default) puts it in flow: it slides in and the base gives up exactly the room it takes, like a sidebar beside the content; it shares its geometry with `NavigationSplitView` and its drawer (`primitives::dock`, through `AdaptiveSplit`), and takes the whole edge (length, align, margin and border are the overlay's). `Overlay` floats it over the base on an [Overlay](dialog.md): the base is faded (`scrim(false)` keeps it clear) and disabled, the surface slides in from its edge on the sheet spring, and a press outside sends `on_outside`.
- **Scope.** `Content` docks to the base as given (a pane, the content area); `Window` docks to the whole window and the corners on the window's corners take its radius (`window_radius`). The base is what you wrap: the window's root for a window dock.
- **Open.** `open(bool)` shows or hides it without moving the base in the widget tree.
- Controlled: the app owns `open` and closes an overlay dock on `on_outside`.

## iced mapping

`widget::Dock` is `AdaptiveSplit` (with `side` and `extent`, the pure geometry in `primitives::dock`: `arrange`, `Flow::Push` / `Flow::Overlay`) for a pushing dock and `widget::Overlay::docked(side, size)` with `along(length, align)` and `margin` for an overlay; the border is drawn around the surface's bounds grown past the flush sides and clipped, so those sides stay open; the panel's inner corners take the theme radius.

## Deviations / gaps

No drag handle or drag to dismiss, no bottom bar, and it animates in only (no slide out).
