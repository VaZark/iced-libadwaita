# MenuButton and menus

libadwaita: [`ComboRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ComboRow.html)

## What it is

Icon button opening a menu: the primary menu (`open-menu-symbolic`, "Main Menu") at the end of the header bar, or a secondary menu. Presented as a popover or as a full page. See [../design-language/menus.md](../design-language/menus.md).

## Anatomy and behavior

- Sections of action and check items, separated by hairlines; accelerators shown dimmed at the end; the primary menu ends with Preferences, Keyboard Shortcuts, Help, About.
- Dropdown: popover (radius 15, padding 8, items radius 9, 32px high) below the button, right-aligned.
- Full page: back button + title header (47px) and 50px rows in boxed lists, one per section.

## CSS nodes and classes

GTK: `menubutton` > `button.toggle` > content; popover is `popover.menu`.

## Spec

Button 34×34 flat; popover width 260; full-page header 47px; back hit area 64×47.

## iced mapping

`primary_menu(menu)`, `menu_button(icon, menu)`, `MenuButton::{presentation, tooltip, window_radius}`, `Menu::new(title).section([MenuItem::action(..), MenuItem::check(..)])`, `MenuPresentation::{Dropdown, FullPage}` with `for_width(w)`.

## Deviations / gaps

No submenus (the HIG forbids nesting), no access keys, no keyboard navigation, no icons-only horizontal button sections.
