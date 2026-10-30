# AlertDialog

libadwaita: [`AlertDialog`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.AlertDialog.html)

## What it is

A message/question dialog: heading, body, optional extra child, responses as buttons (1.5).

## Anatomy and behavior

Responses have id, label, enabled, appearance (default/suggested/destructive); default and close response; buttons laid out horizontally or vertically depending on room (`prefer-wide-layout`).

## CSS nodes and classes

`dialog` (no style classes documented).

## Spec

Heading `.title-2`, body centered, button strip with hairlines; suggested/destructive buttons colored and bold.

## iced mapping

`AlertDialog::new(h).body().response(label, appearance, msg)`.

## Deviations / gaps

Always horizontal; no default/close response or enabled state.
