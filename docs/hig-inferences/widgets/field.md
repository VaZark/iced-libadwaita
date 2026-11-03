# Field

libadwaita has no form-field widget: rows (`AdwEntryRow`, `AdwSpinRow`, ...) carry their own title and the error styling is a bare `.error` class on the entry. `Field` is the HTML `label + control + message` / Radix `Form.Field` pattern for everything else (entries, text views, dropdowns, check and radio buttons, scales, switches), so each form control gets the same layout and the same validation look.

- **Pristine until touched.** Like CSS `:user-valid` / `:user-invalid`, nothing is shown while the user has not interacted; once the field is `touched` (edited or left) or a submit was attempted (`submitted`), it shows warning or invalid. **Only `:user-invalid` (and a warning) is highlighted: `:user-valid` adds nothing special by default**; `Presentation::Valid` is still reported so the app can read it, and `.show_valid(true)` opts in to a success ring. Disabled fields never show validation.
- **Required** is a marker (a red asterisk after the title); it does not validate by itself. `validation::required(value, msg)` produces the error.
- **Message.** The description sits below the control; while a message is shown it replaces the description, in the semantic error (or warning) color with a leading icon. A valid field shows nothing extra unless `.show_valid(true)`.
- **Outline.** One ring (focus-ring width, `radius_m` plus the ring) around the control in the semantic error, warning or success color; transparent when pristine so nothing shifts. Layouts: `Stacked` (title above) and `Row` (title and description at the start, control at the end, message below). In a `Row`, a control that hugs its content keeps its size and the title takes the rest (a long title wraps only where it must); a control that fills shares the row, a third for the title. `.plain_label(true)` draws the title in the body style instead of bold, for plain lists of controls.

## iced mapping

`widget::Field` over `primitives::validation` (pure: `Validity`, `present`, `chain`, `required`; unit tested). The state is controlled: the application passes `validity`, `touched`, `submitted`; the field stores nothing. The control is wrapped in a container that draws the ring, at a fixed position in a column, so wrapped widgets keep their state and their overlays (a `Popover` opens inside a field).

Deviations:

- The ring is a container border around the control, not the control's own border. An `Entry` inside a `Field` is left in its normal state (the field draws the ring); a standalone `Entry` can take the same look with `.presentation(..)` (`EntryState` converts from it).
- Blur is not observable in iced, so `touched` is the application's: usually "has been edited", or set when the control loses focus if the app tracks it.
- Controls without a box (check or radio button, scale) take `.inline(true)` to pad them inside the ring.
- `.disabled(true)` dims the label and hides validation; disabling the control itself is done as everywhere else, by omitting its `on_*` callback.
- The `*Row` widgets already carry a title and are not wrapped; they should show errors through their own style.
- `DateInput` still colors its entry from its own parse check (`text_state`) even while the field is pristine.

- A `Row` field reserves the focus-ring width around its control, so the control sits that much inside the row's end edge (a bare `Switch` in a card is flush); the title stays flush at the start.
