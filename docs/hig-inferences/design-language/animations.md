# Animations

Sources: [`AdwAnimation`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Animation.html),
[`AdwTimedAnimation`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.TimedAnimation.html),
[`AdwSpringAnimation`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SpringAnimation.html),
[`AdwSpringParams`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/struct.SpringParams.html),
[`AdwEasing`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/enum.Easing.html), and the
parameters read from libadwaita's C source and stylesheet (`main`).

## The model

* **`AdwTimedAnimation`**: value `from` → `to` over `duration` ms through an `easing`;
  `repeat-count` (or forever), `reverse` (runs `to` → `from`, time mirrored before easing),
  `alternate` (flip direction every other iteration).
* **`AdwSpringAnimation`**: a physically simulated damped spring (`mass`, `stiffness`,
  `damping` or `damping ratio`): 0 undamped, <1 underdamped (overshoots), 1 critically damped
  (fastest without oscillating), >1 overdamped. Has `initial-velocity` (gesture hand-off),
  `epsilon` (settle threshold, default 0.001), `clamp` (stop on reaching the target) and an
  `estimated-duration` (springs have no fixed duration).
* **`AdwEasing`**: 35 curves: linear; ease-in/out/in-out for quad, cubic, quart, quint, sine,
  expo, circ, elastic, back and bounce; plus (1.7) the CSS cubic-beziers `ease`, `ease-in`,
  `ease-out`, `ease-in-out`.
* **System setting:** animations are skipped when `gtk-enable-animations` is off
  (`follow-enable-animations-setting`, default on) or the widget is unmapped. Sheets use a
  critically damped spring (ratio 1) when the user asks for reduced motion.
* Retargeting an animation mid-flight must keep motion continuous (springs carry their velocity).

## What libadwaita actually uses (from source)

| Where | Animation | Parameters |
|---|---|---|
| `AdwNavigationView` push/pop | spring | damping ratio 1, mass 1, **stiffness 1000** |
| `AdwOverlaySplitView` sidebar, `AdwCarousel` scroll | spring | damping ratio 1, **mass 0.5, stiffness 500**; clamped when hiding |
| `AdwBottomSheet` / `AdwDialog` open/close | spring | **damping ratio 0.8** (1.0 if reduced motion), mass 1, **stiffness 400**, epsilon 0.0001 |
| Bottom-bar reveal | timed | **250ms**, `ADW_EASE` |
| `AdwToast` show (slide up) | timed | **300ms** (500ms when replacing a toast), `ADW_EASE` |
| `AdwToast` hide | timed | 300ms, `ADW_EASE`; opacity eased `EASE_OUT_CUBIC`; scale 0.95 → 1 |
| Widget state transitions (buttons, toggles, switch focus, separators) | CSS | **200ms, `$ease-out-quad`** = `cubic-bezier(0.25, 0.46, 0.45, 0.94)` |
| Scrollbar slider/trough | CSS | 200ms linear |
| Backdrop / focus ring | CSS | 200ms ease-out / `$ease-out-quad` |

Not in the sources I could read (so not asserted): the `GtkRevealer` settings behind
`AdwExpanderRow` / `AdwBanner` (we use the 250ms `ADW_EASE` reveal above) and the switch knob's
own timing (we use the 200ms style transition).

## iced mapping

`iced_libadwaita::animation` mirrors the model above in pure, unit-tested code:
`Easing` (all 35 + `CubicBezier`), `TimedAnimation`, `SpringParams` / `SpringAnimation`
(damped-spring solver, initial velocity, epsilon, clamp, estimated duration), `Tween` (the
animated value a widget keeps; retargeting continues from the current value and velocity),
`presets` (the table above), and `animation::enabled()` / `set_enabled(..)` (follows the
desktop's `enable-animations`; disabled animations jump to the end value).

Widgets animate themselves from iced's `RedrawRequested` events, so the app needs no
subscription or messages:

| Widget | Motion |
|---|---|
| `Switch` | knob slides and track cross-fades, 200ms `$ease-out-quad` |
| `ExpanderRow`, `Banner` (`reveal`) | slide open/closed, 250ms `ADW_EASE` |
| Collapsed-sidebar drawer (`drawer_layer`) | sidebar spring (1, 0.5, 500) slide + scrim fade, **open and closed** |
| `dialog_in_window`, `bottom_sheet_in_window` (`entrance`) | sheet spring (0.8, 1, 400) slide-in + scrim fade |
| `toast_overlay` | slide up, 300ms `ADW_EASE` |
| `carousel` | scroll spring (1, 0.5, 500), drag with velocity hand-off, indicator follows the position |

Limitations: iced has no opacity layer or scaling transform, so dialogs/toasts slide and the
scrim fades but children don't fade or scale; dialogs, bottom sheets and toasts are mounted
only while shown, so they animate **in** but not out; no navigation-view page transition or
swipe gestures; stock iced widgets (button hover, text input focus) can't animate because their
style functions only receive the current status, so only crate-owned widgets (Switch, …) do.

## Triggers

Every action that animates is a `Trigger` with **its own** animation and a `None` fallback.
`Trigger::motion()` resolves to `None` when the desktop disables animations
(`enable-animations`, `[motion] animations=off`) or `[motion] reduced-motion=true`; otherwise it
is the per-trigger `[motion] <key>=default|none` setting. A `None` trigger jumps to its end state
(`Spec::Instant`); widgets still lay out and behave identically.

| Trigger | Config key | Animation |
|---|---|---|
| `PageChange` | `page-change` | content fades in, 200ms ease-out-quad (`AdwViewStack` cross-fade) |
| `PagePush` | `page-push` | page slides in over the old one, which moves a third out and dims; navigation spring (1, 1, 1000) |
| `Popover` | `popover` | rises 8px into place (a full-page menu slides in); 200ms ease-out-quad |
| `Selection` | `selection` | highlight fades in/out, 200ms ease-out-quad |
| `Drawer` | `drawer` | slides from the start edge over a fading scrim; sidebar spring (1, 0.5, 500) |
| `Dialog` | `dialog` | scrim fades in, dialog slides up; sheet spring (0.8, 1, 400) |
| `Sheet` | `sheet` | scrim fades in, sheet slides up; sheet spring (0.8, 1, 400) |
| `Toast` | `toast` | slides up, 300ms ease |
| `Reveal` | `reveal` | slides open/closed, 250ms ease |
| `Switch` | `switch` | knob slides and track cross-fades, 200ms ease-out-quad |
| `Carousel` | `carousel` | scroll spring (1, 0.5, 500) with drag velocity hand-off |
| `Layout` | `layout` | when a breakpoint is crossed (sidebar appearing or disappearing, mobile mode) the new layout fades in from the window background, 200ms ease-out-quad |

An app can override one trigger (discouraged, reported like any opt-out):
`Config::load().opt_out(|o| { o.motion(Trigger::Toast, Motion::None); })`.
