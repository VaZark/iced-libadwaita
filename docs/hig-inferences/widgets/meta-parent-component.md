# MetaParentComponent

No libadwaita or GTK counterpart. A container that **notifies its meta children** to switch mode.

## What it is

Every child registers separately and announces the actions it implements. `registry.notify(&action)` switches exactly the children that announced it:

- a `MetaComponent` (or any component with modes) that did **not** announce the action is unrelated and is left alone;
- a child that never registered, any non-meta widget, is never notified.

## Anatomy and behavior

- **Registration.** `MetaParentComponent::register(&mut registry, id, &meta)` announces a `MetaComponent`'s actions (the keys its variants are selected by, `MetaComponent::actions()`); `registry.register(id, actions)` announces those of any other component, e.g. `ReorderableList::actions()`.
- **Notification.** `notify(&action)` (everyone who announced it), `notify_one(&id, &action)`, `reset()`.
- **Display.** `.meta(id, MetaComponent)` selects the last action it was switched to; `.listener(id, |current| element)` hands it to any other builder; `.child(element)` adds a plain child.
- **Controlled.** The app owns the `Registry` and calls `notify` from `update`; the parent only reads it.

## iced mapping

`primitives::meta_parent::Registry<Id, K>` is pure and unit tested; `MetaParentComponent` lays the children out in a column (or a row). A `MetaComponent` cannot shrink to its content's height (it is built on `responsive`), so give those children a fixed height.
