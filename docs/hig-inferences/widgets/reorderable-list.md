# ReorderableList

libadwaita: rows of a `GtkListBox` with a `GtkDragSource` and a `list-drag-handle-symbolic` grip. Declared like Chakra UI's [Editable](https://chakra-ui.com/docs/components/editable): both views and their controls in one container that owns the switching.

## What it is

A list with a **preview** view (plain rows) and an **edit** view (rows with a grip each). Editing means dragging the rows into a new order.

## Anatomy and behavior

- **Declared once, one message.** `ReorderableList::of(&state, row).on_event(Msg::List)`, and `Msg::List(e) => state.apply(e)` is the whole wiring. `ReorderEvent` is `Edit`, `Reorder { from, to }`, `Submit`, `Cancel`; `ReorderState` keeps the committed order and the draft (Done commits it, Cancel throws it away). One `row` builder serves both views; `edit_row` gives the edit view its own.
- **A second callback.** `on_change(..)` is an independent one for an outside watcher (a `MetaParentComponent`'s `Reporter`) on each drop. Both are published, `on_event` first.
- **Controls** (Chakra's `EditableControls`): Edit in preview; Cancel and Done in edit. `controls(false)` hides the bar; `disabled(true)` dims Edit.
- **Drag.** Press a row's grip. Three things are told apart, in the accent color of the theme: the **held row** is opaque (libadwaita's card color is translucent, so it gets the view color under it), floats above the others with popover elevation and an accent ring, grows a tiny bit (`lift(scale)`, default 1.02, 1.0 for none) and is kept inside the list; the **target slot** (where the drop would land) is tinted in the accent color with a 2px accent line on the edge it is moving toward, like Spotify's insertion line and GTK's `drop-active` row; the **other rows** slide out of the way, eased with the `selection` motion (200ms ease-out-quad; instant under reduced motion). The drop slot comes from the pointer, so the first and last slots are always reachable. Escape puts the row back; the release drops it. Rows keep their own pointer behavior in edit mode.
- **Views as variants.** The two views are picked with `primitives::meta::pick` on `Trigger::selected(Mode::Edit)`, like a `MetaComponent` (called directly, as `Form` does: `responsive` cannot shrink to its content). A `MetaParentComponent` can switch the list: `ReorderableList::actions()` is what it announces.

## iced mapping

`primitives::reorder` (`Mode`, `ReorderState`, `move_item`, `target`, `shift`) is pure and unit tested; the widget's drag is tested through iced events with a null renderer.

Not included: moving a focused row with the keyboard (no focus model), an animated settle when dropped, touch long-press to start a drag.
