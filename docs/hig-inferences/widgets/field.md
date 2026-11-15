# Field

libadwaita has no form-field widget: rows (`AdwEntryRow`, `AdwSpinRow`, ...) carry their own title and the error styling is a bare `.error` class on the entry. `Field` is the HTML `label + control + message` / Radix `Form.Field` pattern for everything else (entries, text views, dropdowns, check and radio buttons, scales, switches), so each form control gets the same layout and the same validation look.

- **Pristine until touched.** Like CSS `:user-valid` / `:user-invalid`, nothing is shown while the user has not interacted; once the field is `touched` (edited or left) or a submit was attempted (`submitted`), it shows warning or invalid. **Only `:user-invalid` (and a warning) is highlighted: `:user-valid` adds nothing special by default**; `Presentation::Valid` is still reported so the app can read it, and `.show_valid(true)` opts in to a success ring. Disabled fields never show validation.
- **Required** is a marker (a red asterisk after the title); it does not validate by itself. `validation::required(value, msg)` produces the error.
- **Message.** The description sits below the control; while a message is shown it replaces the description, in the semantic error (or warning) color with a leading icon. A valid field shows nothing extra unless `.show_valid(true)`.
- **Outline.** With known target geometry, one ring in the semantic error, warning or success color follows the control's four corners and expands by the ring width. Unknown arbitrary controls receive no guessed ring; messages and reserved spacing remain. Pristine validation does not paint, so presentation changes do not shift layout. Layouts: `Stacked` (title above) and `Row` (title and description at the start, control at the end, message below). In a `Row`, a control that hugs its content keeps its size and the title takes the rest (a long title wraps only where it must); a control that fills shares the row, a third for the title. `.plain_label(true)` draws the title in the body style instead of bold, for plain lists of controls.

## iced mapping

`widgets::Field` / `FieldProps` are compatibility exports of `components::InputWrapper` / `InputWrapperProps`, over `primitives::validation`. Validation is controlled: the application passes `validity`, `touched`, `submitted`. The outline wrapper keeps the control's tree position stable so its state and overlays survive validity changes.

Use `target_geometry` to resolve actual corners from the current theme and target bounds, returning `None` when unknown. `target_style` reads corners from the same final container style resolver used by the control, including its controlled state; it does not install or modify that style. `InputWrapper::outline_target` provides the same explicit contract without label/message. These resolvers are target-specific: an ancestor boundary constrains the field subtree but cannot identify the control's silhouette.

The free `outlined` / `input_outline` helper also requires a target geometry resolver; no helper guesses control corners from a theme metric. Default `InputWrapperProps` preserves an existing resolver, presentation and parent-owned validation/events.

Deviations:

- The ring is a separate outline around the resolved target, not a replacement for the control's own border. An `Entry` inside a `Field` is left in its normal state (the field draws the ring); a standalone `Entry` can take the same look with `.presentation(..)` (`EntryState` converts from it).
- Blur is not observable in iced, so `touched` is the application's: usually "has been edited", or set when the control loses focus if the app tracks it.
- Controls without a box (check or radio button, scale) can take `.inline(true)`: the wrapper owns a padded frame with current-theme `radius_m` corners unless explicitly overridden. This is frame geometry, not a guessed control shape.
- `.disabled(true)` dims the label and hides validation; disabling the control itself is done as everywhere else, by omitting its `on_*` callback.
- The `*Row` widgets already carry a title and are not wrapped; they should show errors through their own style.
- `DateInput` still colors its entry from its own parse check (`text_state`) even while the field is pristine.

- A `Row` field reserves the focus-ring width around its control, so the control sits that much inside the row's end edge (a bare `Switch` in a card is flush); the title stays flush at the start.
