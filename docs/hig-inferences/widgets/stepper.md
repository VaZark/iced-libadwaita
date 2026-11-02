# Stepper

libadwaita has no stepper; the GNOME HIG shows multi-step flows as pages in a `AdwCarousel` or `AdwNavigationView` with Back / Next buttons. `Stepper` follows Mantine's [`Stepper`](https://mantine.dev/core/stepper/) (also the pattern behind Material's and Ant's steps): a labelled row of steps, each with a circle (number, icon, check, error or loading mark), joined by lines, plus the active step's content.

- **Controlled `active`.** `0..steps` is a step; `steps` is the completed screen (all steps completed, the `completed` closure shows).
- **Statuses from `active`.** Before it: completed (check or `completed_icon`); at it: current (filled accent); after it: upcoming (outlined). `invalid` (error mark, error color) wins over `loading` (three dots), which wins over the position.
- **Clicks.** `on_step_click(i)` is sent for steps that respond: the active one never does; steps before it always do; steps after it only with `allow_next_steps_select` (default on). Per step, `disabled(true)` (Mantine `allowStepClick={false}`) never responds and `allow_click(true)` always does.
- **Connectors.** The line between two steps fills with the accent up to the active step; `step_progress` fills the line leaving the active one partially.
- **Layout.** Horizontal: the row, then the active step's content. Vertical: steps stacked, the line running down beside the content of the active step. `icon_position` Left or Right of the label; `size` Xs..Xl; `icon_size` overrides the diameter.
- **Content.** Each step's `content` is a closure called only while it is active.
- **Next / back** are the app's buttons; the logic is `primitives::stepper` (`next`, `prev`, `StepperState::forward` / `back` / `goto`).

## iced mapping

`widget::Stepper` over `primitives::stepper` (pure, unit tested: statuses, click rules, next / prev, connector fill). One status type, `Status`, shared with `primitives::form` as `StepStatus`. `form::StepIndicator` is a thin wrapper that hands a `Form`'s derived statuses and reachable steps to `Stepper::statuses` / `clickable`.

Colors are the theme's semantic colors; svg tints are solid mixes (iced ignores a tint's alpha), as in `Entry` sections.

Deviations from Mantine:

- No `color`, `radius`, `iconSize` per CSS token: `size` is a five-step enum and `icon_size` a pixel override.
- `wrap` and `collapse` are additions for narrow widths: by default a horizontal row that does not fit keeps only the current step's label; the fit is estimated from the character count, not measured.
- No `contentPadding`, `autoContentPadding`, `completedIcon` on the stepper (it is per step), `Stepper.Step` as a child component (steps are `StepperStep` values), or `allowStepSelect` (use `disabled`).
- The line is a fixed 2px with a hard-edged fill, not animated (iced has no animation clock here).
- A loading step shows static dots, not a spinner.
