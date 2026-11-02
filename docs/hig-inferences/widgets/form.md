# Form

libadwaita has no form widget; preference pages commit on change and dialogs use a suggested-action response. `Form` is the HTML `<form>` / Radix `Form.Root` pattern for apps that collect several values and submit them at once: it owns **submit, reset and validation gating**, and a native **multi-step wizard** (the GNOME HIG has no wizard either; the nearest pattern is a dialog flow with Back / Next and a suggested-action final button). A single-page form is the one-step case.

- **One step.** Standard chrome: title, description, the fields, and an action bar with Reset (start) and Submit (end, suggested). No step indicator, no Back or Next.
- **Many steps.** A step indicator (numbered, labelled; completed shows a check, the current one is filled with the accent, upcoming ones are outlined, an invalid one carries an error mark), Back, Reset and Next. On the last step Submit replaces Next.
- **Gating.** Next is refused while the current step has an error (a warning never blocks). A refused Next, or a refused Submit, **reveals** the step's invalid fields, like pressing Submit on an HTML form with `:user-invalid`; the app gets the first invalid key to focus. Back is always allowed. A click on the indicator goes back freely; forward only when every earlier step is valid (otherwise it lands on the first invalid step, attempted). Submit with an error on an earlier step jumps to that step.
- **Reveal rule.** A field shows its validity if it was touched, or its step was attempted, or the form was submitted. Only invalid (and warnings) draw anything; a valid field adds nothing unless the field opts in (see [Field](field.md)).
- **Reset** clears touched, attempted, submitted and returns to step one; the app resets its values.
- **Enter** triggers the primary action (Next, or Submit on the last step) unless a focused text input handled it.

## iced mapping

`widget::Form` over `primitives::form` (pure: `FormSpec`, `FormState`, `Outcome`, `StepStatus`; unit tested). Controlled: the app owns the values and one `FormState<K>` (K is its field key) and calls `touch`, `next`, `back`, `goto`, `submit`, `reset` from `update`, so the buttons only emit the app's messages (`on_submit`, `on_reset`, `on_next`, `on_back`, `on_step`). `FormState::interaction(spec, key)` feeds `Field::interaction`, so a field needs no `touched` / `submitted` bookkeeping. `Form::state(state, spec, validity_of)` precomputes each step's indicator state and which steps a click may reach.

Deviations:

- Only the current step's content is built (a closure per `Step`), the same choice as a `MetaComponent` with `Trigger::selected(step)` and through the same `meta::pick`; `MetaComponent` itself is not used because it sits on iced's `responsive`, which cannot take its content's height inside a scrolled column.
- `submitting(true)` disables the buttons and relabels Submit "Submitting…"; there is no spinner (iced has no animation clock; the app can show a `Spinner` itself).
- Blur is not observable in iced, so `touched` is the app's (usually "edited").
- The step indicator does not wrap or collapse its labels on a very narrow width.
- Focusing the first invalid field is left to the app (the outcome carries the key); the toolkit entries take no widget id yet.
