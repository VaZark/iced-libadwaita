# DropDown

GTK4: [`GtkDropDown`](https://docs.gtk.org/gtk4/class.DropDown.html)

## What it is

Choose one item from a model. Shows the current selection and a popup list; optional search entry, header factory, custom item factory.

## Behavior

`selected` / `selected-item`, `show-arrow` (4.6), `enable-search`, `list-factory`.

## CSS nodes

```
dropdown
├── button
╰── popover
```

## Style classes and states

None documented. libadwaita styles the popover as `.menu`.

## Adwaita defers

Adwaita's popover surface, 12px radius, selected item marked with a trailing check; `AdwComboRow` embeds it. See [../widgets/dropdown.md](../widgets/dropdown.md).

## iced mapping

`adw::widget::dropdown` / `Dropdown`.
