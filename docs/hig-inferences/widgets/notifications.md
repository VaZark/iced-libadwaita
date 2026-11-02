# Notifications queue

The state behind a stack of [toasts](toast.md): which are on screen, how many, and when each leaves. Libadwaita has one toast at a time (`AdwToastOverlay`) and an app-side queue; Mantine's notifications store (`notifications.show / update / hide / clean`, `limit`, `autoClose`) is the model here, so the apps do not each re-write it.

#### Shape

`primitives::notifications::Queue<T>` (pure, headless, unit-tested; the app owns it) and `widget::toast_stack_from(content, &queue, |id, item| Toast)` (draws it with `toast_stack`).

| Operation                      | Meaning                                                                                                                          | Mantine         |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------- | --------------- |
| `push(value, now) -> Id`       | add; above `limit` (default 4) the oldest is dropped; ids are never reused                                                       | `show`, `limit` |
| `replace(id, value, now)`      | swap the content in place (a toast whose action ran becomes a confirmation); restarts its timer; `false` if it is gone           | `update`        |
| `dismiss(id)` / `clear()`      | remove by hand                                                                                                                   | `hide`, `clean` |
| `timeout(Some(d))`             | each notification leaves `d` after it was pushed or replaced; `None` (default) keeps it                                          | `autoClose`     |
| `expire(now)`, `next_expiry()` | remove the due ones; `next_expiry()` is `None` when nothing can expire, so the app subscribes to a clock only while it is `Some` | --              |

#### iced mapping

- The time is explicit (`Instant` in, nothing read from a clock), so the tests need no sleeping; the app maps `iced::time::every(..)` to `expire(now)`.
- The toast's own dismiss and action messages carry the `Id`.
- Hover does not pause the timer (Mantine pauses on hover); not implemented.
- No per-notification timeout and no `loading` state; one queue-wide timeout.
- Gallery: Feedback / Toast (Stacked notifications, Auto-dismiss).
