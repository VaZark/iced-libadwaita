# ReorderableList

libadwaita: rows of a `GtkListBox` with a `GtkDragSource` and a `list-drag-handle-symbolic` grip. API after Chakra UI's [Editable](https://chakra-ui.com/docs/components/editable).

## What it is

A list with a **preview** view (plain rows) and an **edit** view (rows with a grip each). Editing means dragging the rows into a new order.

## Anatomy and behavior

- **Controlled.** The app passes `editing(bool)`; `ReorderState` (the `reorder` primitive) holds the committed order and the draft that edit mode works on. Edit starts a draft, `on_reorder(from, to)` changes it, **Done** (`on_submit`) commits it, **Cancel** (`on_cancel`) throws it away.
- **Controls** (Chakra's `EditableControls`): Edit in preview; Cancel and Done in edit. `controls(false)` hides the bar; `disabled(true)` dims Edit.
- **Drag.** Press a row's grip: the row lifts (popover elevation) and follows the pointer, kept inside the list; the rows between its slots move out of the way; the release drops it in the slot its center is over. Escape puts it back. Rows keep their own pointer behavior in edit mode.
- **Views as variants.** The two views are picked with `primitives::meta::pick` on `Trigger::selected(Mode::Edit)`, like a `MetaComponent` (called directly, as `Form` does: `responsive` cannot shrink to its content). A `MetaParentComponent` can switch the list: `ReorderableList::actions()` is what it announces.

## iced mapping

`primitives::reorder` (`Mode`, `ReorderState`, `move_item`, `target`, `shift`) is pure and unit tested; the widget's drag is tested through iced events with a null renderer.

Not included: moving a focused row with the keyboard (no focus model), an animated settle when dropped, touch long-press to start a drag.
