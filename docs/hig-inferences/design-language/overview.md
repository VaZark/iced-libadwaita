# Overview

* **Light and dark are both first class.** Apps default to light unless the
  system prefers dark (`AdwColorScheme`: force-light, prefer-light,
  prefer-dark, force-dark). Custom drawing must use the CSS variables, never
  hard-coded colors.
* **Accent follows the system** (nine accents). Custom UI uses
  `--accent-bg-color` / `--accent-fg-color` / `--accent-color`; never hard-code
  blue. For a bright custom accent set the foreground to dark (80% black).
* **High contrast** is a separate system preference that works with dark.
  Custom styling should use the helper opacities (`--border-opacity`,
  `--dim-opacity`, `--disabled-opacity`) instead of fixed alphas; they change
  under high contrast (15/55/50% → 50/90/40%).
* **Surfaces, not colors.** Widgets ask for a role: window, view, card, dialog,
  popover, headerbar, sidebar, secondary sidebar, overview, thumbnail, active
  toggle. Each has a bg/fg pair.
* **Translucent controls.** Default buttons/entries are tints of the
  foreground (`alpha(currentColor, x)`), not fixed grays, so they work on any
  surface.
* **Semantic color groups** (destructive, success, warning, error, accent) each
  come as `bg`, `fg` and a standalone `color` for text on a view. The
  standalone color is derived from the bg in OKLab (light: lightness ≤ 0.5,
  dark: lightness ≥ 0.85).
* **Adaptive by construction.** Nothing has a fixed minimum width: windows
  declare a minimum (360×200) and breakpoints reflow the UI. Labels must
  ellipsize or wrap; buttons use `can-shrink`.
* **Client-side decorations only.** Header bars are part of the app; there is
  no server-side titlebar. See [windows-and-csd.md](windows-and-csd.md).
