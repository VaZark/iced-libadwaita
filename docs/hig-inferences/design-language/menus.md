# Menus

Sources: [GNOME HIG: Menus](https://developer.gnome.org/hig/patterns/controls/menus.html),
[HIG: Header Bars](https://developer.gnome.org/hig/patterns/containers/header-bars.html),
[Material: Menus](https://m2.material.io/go/design-menus), and libadwaita's
[`AdwDialog:presentation-mode`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/property.Dialog.presentation-mode.html).

## GNOME HIG

- **Primary menu:** `open-menu-symbolic` button, tooltip/label "Main Menu", at the
  **end** of the header bar. With a sidebar it goes **above the sidebar list** (in
  the sidebar header). With hierarchical navigation show it only on the top-level
  view; other views use a secondary menu.
- **Standard items go in a group at the end:** Preferences, Keyboard Shortcuts,
  Help, **About [App Name]** (every primary menu has About). **No Close/Quit.**
- **Items:** verbs for commands, adjectives for settings, header capitalization;
  an access key for each; checkboxes/radios allowed.
- **Size:** 3–12 items (submenus 3–6); order by importance/frequency; group similar
  items; do **not** nest submenus; don't mix actions and check items in one group.
- **Secondary menus:** hold actions for the current view/content; don't repeat
  primary items.
- **Header bar:** start = main actions (new, add, open, back); center = window
  heading (or view switcher); end = menus. Flat icon buttons; avoid label-only,
  suggested/destructive and linked buttons in the primary header.
- The HIG says **nothing** about menus on narrow/mobile screens.

## Narrow / mobile conventions

- Material: on compact screens a **bottom sheet can replace an inline menu**; on
  larger screens keep the anchored context menu so the action stays near its
  source.
- libadwaita `AdwDialog`: presentation **auto** switches from a floating window to
  a **bottom sheet at `max-width: 450` or `max-height: 360`**.
- A full-screen "menu page" for hamburger menus on phones is common mobile
  practice (not from the sources above).

## Decision: two presentations

| `MenuPresentation` | What                                                                                                                            | Use when      |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `Dropdown`         | anchored popover (260px, 9px item radius, hairline between sections, accelerators dimmed at the end)                            | desktop       |
| `FullPage`         | the menu replaces the view: header (back button, centered title) and large 50px rows in boxed lists per section, clamped to 600 | narrow/mobile |

`MenuPresentation::for_width(w)` returns `FullPage` in mobile mode (`w` below 576; libadwaita's dialog threshold is 450, we switch earlier)
(`window::MOBILE_MAX_WIDTH`), else `Dropdown`. The app can also force either.

## iced mapping

`primary_menu(Menu)` / `menu_button(icon, Menu)` with `.presentation(..)`,
`.tooltip(..)`, `.window_radius(..)`. Menus are built from `MenuItem::action` /
`MenuItem::check` (`.accel`, `.icon`, `.destructive`) grouped with `Menu::section`.
Built on the same overlay as `Dropdown`: click outside, Escape or a selection closes
a dropdown; the full page closes on Escape, selection, or its back area. No
submenus (HIG), no access keys, no keyboard navigation yet.
