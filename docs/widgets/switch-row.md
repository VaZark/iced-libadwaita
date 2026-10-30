# SwitchRow

libadwaita: [`SwitchRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SwitchRow.html)

## What it is

An action row with an embedded switch for an on/off setting.

## Anatomy and behavior

Activating the row inverts `active`; the user can also drag the switch. Role: switch. Since 1.4.

## CSS nodes and classes

Inherits ActionRow nodes (no extra documented).

## Spec

Switch ≈46×26 with a 20px knob ‡. Switch sits in the suffix, vertically centered.

## iced mapping

`SwitchRow::new(title, active, on_toggle)`; whole row is activatable. The switch is a custom `adw::widget::Switch` with Adwaita proportions (46×26) rather than iced's 2:1 toggler.
