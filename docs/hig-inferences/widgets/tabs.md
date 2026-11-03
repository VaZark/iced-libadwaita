# Tabs

No libadwaita counterpart for the panels (`AdwTabBar` is the window-level tab strip, see [TabBar](tab-bar.md)). After Mantine's [Tabs](https://mantine.dev/core/tabs/) and Radix UI's.

## What it is

A row of tabs and the panel of the active one.

## Anatomy and behavior

- **Controlled.** `Tabs::new(Some(key), on_change)`; the app owns the active key. Each `TabItem::new(key, label)` may have an `icon`, be `disabled`, and carry a `panel(|| ..)` that is built only while the tab is active. Without panels it is just the list (the gallery's Preview / Code tabs).
- **Variants**, as in Mantine: `Default` (an accent line under the active tab over a hairline), `Outline` (the active tab boxed), `Pills` (filled with the accent color) and `Floating` (a highlight that slides behind the active tab, the [FloatingIndicator](floating-indicator.md)). `grow(true)` shares the width equally (not for `Floating`).
- **Closable.** `TabItem::closable(true)` adds a close button in the style of the variant, and `Tabs::on_close(|key| ..)` receives the tab's key; the app removes the tab.
- **Same behavior as the segmented control.** Which tab a click activates, and that a disabled tab does not, is the `primitives::tabs::TabList`, which `toggle_group` also uses; the primitive has `step`, `first` and `last` for arrow keys and Home / End.

## iced mapping

`primitives::tabs` is pure and unit tested; `widget::Tabs` builds the list from iced buttons (and a `FloatingIndicator` for the floating variant). Not included: arrow-key navigation (iced has no focus model for it), vertical orientation, and keeping the panels of inactive tabs mounted.
