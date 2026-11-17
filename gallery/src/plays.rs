//! The playgrounds of the gallery's component pages (see `playground.rs`): for each page, the
//! controls (`specs`) and the live component built from their values (`build`). A page that is not here shows its own demo: it needs more than a single
//! instance can express (async states, a wizard flow, app-wide theme settings, a layout that
//! depends on the container).

use super::*;
use crate::playground::{Msg as PMsg, Spec};
use iced::widget::{column, container, row};

/// The icon of a playground's icon choice: none, system icons, or an application SVG.
fn icon_source(choice: usize) -> Option<iad::IconSource> {
    match choice {
        1 => Some(Icon::Check.into()),
        2 => Some(Icon::Star.into()),
        3 => Some(iad::IconSource::svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 2l5 6-5 6-5-6z"/></svg>"#,
        )),
        _ => None,
    }
}

/// A preview sits in the playground's plain frame: no card of its own.
fn stage<'a>(e: impl Into<Element<'a>>) -> Element<'a> {
    e.into()
}

fn on_bool(d: Detail, k: &'static str) -> impl Fn(bool) -> Message {
    move |v| Message::Play(d, PMsg::Bool(k, v))
}
fn on_num(d: Detail, k: &'static str) -> impl Fn(f32) -> Message {
    move |v| Message::Play(d, PMsg::Number(k, v))
}
fn on_pick(d: Detail, k: &'static str) -> impl Fn(usize) -> Message {
    move |v| Message::Play(d, PMsg::Choice(k, v))
}
fn on_text(d: Detail, k: &'static str) -> impl Fn(String) -> Message {
    move |v| Message::Play(d, PMsg::Text(k, v))
}

const ROLES: [&str; 5] = ["Accent", "Success", "Warning", "Error", "Destructive"];
const ROLE_OF: [Role; 5] = [
    Role::Accent,
    Role::Success,
    Role::Warning,
    Role::Error,
    Role::Destructive,
];

pub fn specs(d: Detail) -> Vec<Spec> {
    use Spec as S;
    match d {
        Detail::Button => vec![
            S::choice(
                "style",
                "Style",
                &[
                    "Default",
                    "Flat",
                    "Suggested",
                    "Destructive",
                    "Pill",
                    "Suggested pill",
                ],
                0,
            ),
            S::text("label", "Label", "Button"),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::ButtonContent => vec![
            S::text("label", "Label", "Open"),
            S::choice("icon", "Icon", &["None", "Home", "Add", "Search"], 1),
            S::choice(
                "style",
                "Style",
                &["Default", "Suggested", "Circular suggested"],
                0,
            ),
            S::number("circ_size", "Circular diameter (px)", 24.0..=64.0, 34.0),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::ToggleGroup => vec![
            S::choice("selected", "Selected", &["One", "Two", "Three"], 0),
            S::bool("icons", "Icons", false),
            S::bool("fill", "Fill the width", false),
        ],
        Detail::LinkButton => vec![
            S::text("label", "Label", "Link"),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::TextInput => vec![
            S::text("value", "Value", "Hello"),
            S::choice(
                "state",
                "Validation",
                &["None", "Error", "Warning", "Success"],
                0,
            ),
            S::bool("password", "Password", false),
            S::bool("disabled", "Disabled", false),
            S::bool("fixed", "Fixed width (240px)", false),
            S::choice(
                "color",
                "Color override",
                &["None", "Purple", "Teal", "Orange"],
                0,
            ),
        ],
        Detail::Switch => vec![
            S::bool("active", "Active", true),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::CheckButton => vec![
            S::text("label", "Label", "Check me"),
            S::bool("active", "Checked", true),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::RadioButton => vec![
            S::choice("selected", "Selected", &["A", "B", "C"], 1),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::Scale => vec![
            S::bool("vertical", "Vertical", false),
            S::slider("value", "Value", 0.0..=100.0, 40.0),
            S::bool("readout", "Show the value", true),
        ],
        Detail::Dropdown => vec![
            S::choice(
                "selected",
                "Selected",
                &["None", "Apple", "Banana", "Cherry"],
                0,
            ),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::ActionRow => vec![
            S::text("title", "Title", "Ada Lovelace"),
            S::bool("subtitle", "Subtitle", true),
            S::bool("prefix", "Prefix avatar", false),
            S::bool("suffix", "Suffix button", false),
            S::bool("chevron", "Chevron", false),
        ],

        Detail::TreeRow => vec![
            S::bool("expanded", "Expanded", true),
            S::bool("selected", "Selected", false),
            S::bool("subtitle", "Subtitle", true),
            S::bool("prefix", "Icon prefix", false),
            S::bool("suffix", "Suffix", false),
            S::choice(
                "children",
                "Children",
                &["Rows", "Mixed elements", "Nested tree rows"],
                0,
            ),
            S::number("count", "Count", 1.0..=4.0, 2.0),
            S::number("indent", "Indent", 0.0..=60.0, 24.0),
        ],
        Detail::ButtonRow => vec![
            S::text("title", "Title", "Delete account"),
            S::choice("role", "Role", &["Neutral", "Suggested", "Destructive"], 2),
        ],
        Detail::Card => vec![
            S::number("children", "Children", 1.0..=4.0, 2.0),
            S::number("padding", "Padding", 0.0..=32.0, 12.0),
            S::bool("separators", "Separators", true),
        ],
        Detail::Clamp => vec![S::number("max", "Maximum width", 120.0..=600.0, 280.0)],
        Detail::StatusPage => vec![
            S::text("title", "Title", "No Results"),
            S::text("description", "Description", "Try a different search term."),
            S::choice("icon", "Icon", &["Search", "Home", "Settings"], 0),
            S::bool("action", "Action button", true),
        ],
        Detail::WrapBox => vec![
            S::number("count", "Tags", 1.0..=12.0, 7.0),
            S::number("spacing", "Spacing", 0.0..=24.0, 6.0),
        ],
        Detail::Collapse => vec![
            S::choice("behavior", "Collapse behavior", &["Stacked", "Flex"], 0),
            S::slider("value", "Value", 0.0..=100.0, 50.0),
        ],
        Detail::Tree => vec![
            S::bool("lines", "Connecting lines", true),
            S::number("offset", "Level offset", 8.0..=40.0, 20.0),
            S::bool("icons", "Icons", true),
            S::bool("checkboxes", "Checkboxes", false),
            S::bool("strict", "Check strictly", false),
            S::bool("multiple", "Multiple selection", false),
            S::bool("expand_on_click", "Expand on click", true),
            S::text("search", "Search", ""),
            S::choice(
                "search_mode",
                "Search mode",
                &["Off", "Highlight", "Filter", "Fuzzy filter"],
                0,
            ),
            S::choice(
                "drag",
                "Drag and drop",
                &["Off", "By the row", "By a grip"],
                0,
            ),
            S::bool("locked", "Lock the “locked” folder", false),
        ],
        Detail::Container => vec![
            S::choice(
                "size",
                "Size",
                &["Xs 540", "Sm 720", "Md 960", "Lg 1140", "Xl 1320", "Custom"],
                2,
            ),
            S::number("custom", "Custom width (px)", 200.0..=1400.0, 800.0),
            S::bool("fluid", "Fluid (no maximum)", false),
            S::number("padding", "Horizontal padding (px)", 0.0..=48.0, 12.0),
            S::number("parent", "Parent width (px)", 300.0..=1500.0, 1400.0),
        ],
        Detail::Divider => vec![
            S::choice("orientation", "Orientation", &["Horizontal", "Vertical"], 0),
            S::text("label", "Label", "Label"),
            S::choice(
                "position",
                "Label position",
                &["Left", "Center", "Right"],
                1,
            ),
            S::choice("variant", "Variant", &["Solid", "Dashed", "Dotted"], 0),
            S::number("size", "Size", 1.0..=8.0, 1.0),
            S::choice(
                "tone",
                "Color",
                &["Border", "Card", "Bar", "Popover", "Sidebar"],
                0,
            ),
            S::number("spacing", "Spacing", 0.0..=24.0, 8.0),
        ],
        Detail::Splitter => vec![
            S::choice("orientation", "Orientation", &["Horizontal", "Vertical"], 0),
            S::choice("panes", "Panes", &["Two", "Three"], 0),
            S::bool("fixed", "First pane fixed (240 px)", false),
            S::bool("limits", "First pane limits (120 px to 60%)", false),
            S::bool("collapsible", "First pane collapsible", false),
            S::choice("redistribute", "Redistribute", &["Adjacent", "Nearest"], 0),
            S::bool("grip", "Grip", true),
            S::number("line", "Line size", 1.0..=6.0, 1.0),
        ],
        Detail::Docked => vec![
            S::bool("open", "Open", true),
            S::choice("side", "Side", &["Bottom", "Top", "Left", "Right"], 0),
            S::choice(
                "enter",
                "Enter effect",
                &["Slide", "Fade", "None"],
                0,
            ),
            S::choice("exit", "Exit effect", &["Slide", "Fade", "None"], 0),
            S::number("starting", "Starting value (%)", 0.0..=100.0, 0.0),
            S::number("ending", "Ending value (%)", 0.0..=100.0, 0.0),
            S::number("duration", "Duration (ms)", 50.0..=5000.0, 1500.0),
            S::choice(
                "easing",
                "Easing",
                &["Default (spring)", "Ease in-out cubic", "Linear"],
                1,
            ),
            S::choice(
                "size",
                "Size (across)",
                &["Its own", "120 px", "40%", "Full screen"],
                0,
            ),
            S::choice(
                "length",
                "Length (along)",
                &["Whole edge", "320 px", "50%"],
                0,
            ),
            S::choice(
                "align",
                "Position along the edge",
                &["Start", "Center", "End"],
                0,
            ),
            S::choice("mode", "Mode", &["Push content", "Overlay"], 0),
            S::choice("scope", "Scope", &["Content", "Window"], 0),
            S::number(
                "margin",
                "Margin off the edge (0 is flush)",
                0.0..=32.0,
                0.0,
            ),
            S::bool("border", "Hairline border (not on flush sides)", true),
            S::bool("scrim", "Fade the base (overlay)", true),
        ],
        Detail::NumberInput => vec![
            S::number("decimals", "Decimals (0 is an integer)", 0.0..=4.0, 0.0),
            S::bool("limits", "Limits (0 to 100)", true),
            S::choice("step", "Step", &["1", "0.01", "0.1", "0.25", "5", "10"], 0),
            S::choice(
                "drag",
                "Dragging",
                &["Scrub vertically", "Scrub horizontally", "Select text"],
                0,
            ),
            S::bool("handles", "Handles", true),
            S::bool("disabled", "Disabled", false),
        ],
        Detail::ColorPicker => vec![
            S::choice("kind", "As", &["Picker", "Input"], 0),
            S::bool("alpha", "Alpha slider", false),
            S::choice(
                "format",
                "Format",
                &["Hex", "Hexa", "Rgb", "Rgba", "Hsl", "Hsla", "Hsv", "Hsva"],
                0,
            ),
            S::bool("swatches", "Swatches", true),
            S::bool("readout", "Readout (picker)", true),
            S::number("width", "Width (picker)", 160.0..=320.0, 240.0),
            S::bool("dialog", "Open in a dialog (input)", false),
            S::bool("scrim", "Fade the window (dialog)", true),
        ],
        Detail::Timeline => vec![
            S::number("active", "Active item", 0.0..=5.0, 2.0),
            S::number("count", "Items", 2.0..=6.0, 4.0),
            S::choice("align", "Align", &["Left", "Right"], 0),
            S::bool("reverse", "Reverse active", false),
            S::number("bullet", "Bullet size", 16.0..=40.0, 24.0),
            S::number("line_width", "Line width", 1.0..=6.0, 2.0),
            S::choice("variant", "Line", &["Solid", "Dashed", "Dotted"], 0),
            S::bool("loading", "Last item loading", false),
            S::bool("invalid", "Second item failed", false),
        ],
        Detail::Spoiler => vec![
            S::choice("mode", "Part of", &["The height", "The data"], 0),
            S::number("max_height", "Maximum height", 40.0..=240.0, 80.0),
            S::bool("expanded", "Expanded", false),
            S::bool("preview", "Preview (no control, always clipped)", false),
            S::number("duration", "Open/close duration (ms)", 50.0..=5000.0, 250.0),
            S::choice("width", "Width", &["Fill", "Fixed 240 px"], 0),
            S::number("initial", "Items shown first", 1.0..=10.0, 3.0),
            S::choice("step", "A press reveals", &["All", "5 more", "10 more"], 2),
        ],
        Detail::NoteCard => vec![
            S::bool("selected", "Selected", false),
            S::choice("tint", "Tint", &["None", "Red", "Green", "Blue"], 1),
            S::number("max_height", "Body height", 40.0..=200.0, 70.0),
        ],
        Detail::Tabs => vec![
            S::choice(
                "variant",
                "Variant",
                &["Default", "Outline", "Pills", "Floating"],
                0,
            ),
            S::choice(
                "selected",
                "Active tab",
                &["Gallery", "Messages", "Settings"],
                0,
            ),
            S::bool("grow", "Grow", false),
            S::bool("icons", "Icons", false),
            S::bool("disabled", "Settings disabled", false),
            S::bool("panels", "Panels", true),
            S::bool("closable", "Closable tabs", false),
            S::bool("show_messages", "Messages tab", true),
            S::bool("show_settings", "Settings tab", true),
        ],
        Detail::ViewSwitcher => vec![S::choice(
            "selected",
            "Selected",
            &["Files", "Search", "Settings"],
            0,
        )],
        Detail::PathBar => vec![S::choice(
            "current",
            "Current segment",
            &["Home", "Projects", "iced-adaptive-design"],
            2,
        )],
        Detail::ShortcutLabel => vec![S::text("keys", "Keys", "Ctrl+Shift+P")],
        Detail::Banner => vec![
            S::text("title", "Title", "Network connection lost"),
            S::bool("button", "Button", true),
            S::bool("revealed", "Revealed", true),
        ],
        Detail::Progress => vec![
            S::slider("value", "Value", 0.0..=100.0, 60.0),
            S::choice(
                "role",
                "Style",
                &["Default", "Success", "Warning", "Error"],
                0,
            ),
        ],
        Detail::ClipWindow => vec![
            S::number("height", "Window height (px)", 0.0..=220.0, 90.0),
            S::choice("anchor", "Anchor", &["Top", "Bottom"], 1),
            S::bool("compare", "Compare with a squeezed container", true),
        ],
        Detail::IconToggle => vec![
            S::choice(
                "icon",
                "Icon",
                &["Shuffle", "Repeat", "Heart", "Star", "Queue"],
                0,
            ),
            S::bool("active", "On", false),
            S::bool("disabled", "Disabled", false),
            S::bool("tooltip", "Tooltip", true),
            S::number("size", "Icon size (px)", 12.0..=32.0, 16.0),
        ],
        Detail::Avatar => vec![
            S::text("name", "Name", "Ada Lovelace"),
            S::number("size", "Size", 24.0..=96.0, 48.0),
            S::bool("initials", "Show initials", true),
            S::choice(
                "icon",
                "Icon (replaces the initials)",
                &["None", "Check", "Star", "Own SVG"],
                0,
            ),
            S::choice("shape", "Shape", &["Circle", "Rounded", "Square"], 0),
        ],
        Detail::Tag => vec![
            S::text("label", "Label", "Accent"),
            S::choice("role", "Role", &ROLES, 0),
        ],
        Detail::BadgeChip => vec![
            S::choice(
                "kind",
                "Kind",
                &["Badge", "Count badge", "Chip", "Round badge"],
                0,
            ),
            S::text("label", "Label", "New"),
            S::choice(
                "icon",
                "Icon (chip, round badge)",
                &["None", "Check", "Star", "Own SVG"],
                0,
            ),
            S::choice(
                "color",
                "Color (chip, round badge)",
                &["From the role", "Own color"],
                0,
            ),
            S::choice("chip_size", "Chip size", &["Small", "Medium", "Large"], 1),
            S::bool("on_color", "White text when selected (own color)", false),
            S::choice(
                "contrast",
                "Unselected text contrast (own color)",
                &["Ensure 4.5", "Off (as given)"],
                0,
            ),
            S::choice(
                "text",
                "Unselected text color (own color)",
                &["The chip's color", "Custom (dark gray)"],
                0,
            ),
            S::choice("tint", "Tag color", &["Pink", "Dark blue", "Amber"], 0),
            S::number("size", "Round badge size (px)", 14.0..=48.0, 24.0),
            S::choice("role", "Role", &ROLES, 0),
            S::bool("solid", "Solid", false),
            S::number("count", "Count", 0.0..=150.0, 7.0),
            S::bool("selected", "Selected (chip)", false),
            S::bool("disabled", "Disabled (chip)", false),
        ],
        _ => Vec::new(),
    }
}

/// The live component for `d` built from the playground's values.
pub fn build<'a>(
    g: &'a Gallery,
    d: Detail,
    p: &'a crate::playground::Playground,
    m: Metrics,
) -> Option<Element<'a>> {
    let lbl = |s: &str| -> Element<'a> { cmp::typography::label(s.to_string()).into() };
    let noop = Message::Noop;
    Some(match d {
        Detail::Button => {
            let style = p.choice("style");
            let label = lbl(p.text("label"));
            let press = (!p.bool("disabled")).then_some(Message::Noop);
            let button: Element<'a> = match style {
                0 => cmp::raised_button(label).on_press_maybe(press).into(),
                1 => cmp::flat_button(label).on_press_maybe(press).into(),
                2 => cmp::suggested_button(label).on_press_maybe(press).into(),
                3 => cmp::destructive_button(label).on_press_maybe(press).into(),
                4 => cmp::pill_button(label).on_press_maybe(press).into(),
                _ => cmp::suggested_pill_button(label)
                    .on_press_maybe(press)
                    .into(),
            };
            button
        }
        Detail::ButtonContent => {
            let icon = [
                None,
                Some(Icon::Home),
                Some(Icon::ListAdd),
                Some(Icon::Search),
            ][p.choice("icon")];
            let mut content = cmp::ButtonContent::new(p.text("label").to_string()).metrics(m);
            if let Some(i) = icon {
                content = content.icon(i);
            }
            let press = (!p.bool("disabled")).then_some(Message::Noop);
            let button: Element<'a> = if p.choice("style") == 2 {
                cmp::suggested_circular_button(icon.unwrap_or(Icon::Play).view(m.icon_size))
                    .size(p.number("circ_size"))
                    .on_press_maybe(press)
                    .into()
            } else if p.choice("style") == 1 {
                cmp::suggested_button(Element::from(content))
                    .on_press_maybe(press)
                    .into()
            } else {
                cmp::raised_button(Element::from(content))
                    .on_press_maybe(press)
                    .into()
            };
            button
        }
        Detail::ToggleGroup => {
            let names = [
                ("One", Icon::Home),
                ("Two", Icon::Search),
                ("Three", Icon::Settings),
            ];
            let icons = p.bool("icons");
            let items: Vec<cmp::ToggleItem<'a, usize, Message>> = names
                .iter()
                .enumerate()
                .map(|(i, (n, ic))| {
                    let it = cmp::ToggleItem::new(i, *n);
                    if icons { it.icon(*ic) } else { it }
                })
                .collect();
            let mut tg =
                cmp::toggle_group(items, Some(p.choice("selected")), on_pick(d, "selected"));
            if p.bool("fill") {
                tg = tg.width(Length::Fill);
            }
            Element::from(tg)
        }
        Detail::LinkButton => {
            let b = cmp::link_button(p.text("label").to_string());
            let b: Element<'a> = b
                .on_press_maybe((!p.bool("disabled")).then_some(noop))
                .into();
            b
        }
        Detail::TextInput => {
            let state = [
                None,
                Some(cmp::EntryState::Error),
                Some(cmp::EntryState::Warning),
                Some(cmp::EntryState::Success),
            ][p.choice("state")];
            let value = p.text("value").to_string();
            let el: Element<'a> = if p.bool("password") {
                let mut e = cmp::password_entry("Password", &value).metrics(m);
                if !p.bool("disabled") {
                    e = e.on_input(on_text(d, "value"));
                }
                e.into()
            } else {
                let mut e = cmp::entry("Placeholder", &value).metrics(m);
                if !p.bool("disabled") {
                    e = e.on_input(on_text(d, "value"));
                }
                if let Some(s) = state {
                    e = e.state(s);
                }
                if p.bool("fixed") {
                    e = e.width(Length::Fixed(240.0));
                }
                if let Some(c) =
                    [None, Some(0x9141ac), Some(0x2190a4), Some(0xe66100)][p.choice("color")]
                {
                    e = e.color(iad::color::hex(c));
                }
                e.into()
            };
            el
        }
        Detail::Switch => {
            let mut s = cmp::switch(p.bool("active"));
            if !p.bool("disabled") {
                s = s.on_toggle(on_bool(d, "active"));
            }
            s.into()
        }
        Detail::CheckButton => {
            let mut c = cmp::check_button(p.text("label").to_string(), p.bool("active"));
            if !p.bool("disabled") {
                c = c.on_toggle(on_bool(d, "active"));
            }
            c.into()
        }
        Detail::RadioButton => {
            let sel = Some(p.choice("selected"));
            let mk = |i: usize, name: &str| -> Element<'a> {
                if p.bool("disabled") {
                    cmp::radio_button(name.to_string(), i, sel, |_| Message::Noop).into()
                } else {
                    cmp::radio_button(name.to_string(), i, sel, on_pick(d, "selected")).into()
                }
            };
            let el =
                column![mk(0, "Radio A"), mk(1, "Radio B"), mk(2, "Radio C")].spacing(m.space_s);
            el.into()
        }
        Detail::Scale => {
            let v = p.number("value");
            let vertical = p.bool("vertical");
            let axis = if vertical {
                iad::primitives::scrub::Axis::Vertical
            } else {
                iad::primitives::scrub::Axis::Horizontal
            };
            let mut col = column![
                cmp::scale(0.0..=100.0, v, on_num(d, "value"))
                    .orientation(axis)
                    .height(180.0)
            ]
            .spacing(m.space_s);
            if p.bool("readout") {
                col = col.push(ty::dim_label(format!("value {v:.0}")));
            }
            col.into()
        }
        Detail::Dropdown => {
            let sel = p.choice("selected");
            let el: Element<'a> = if p.bool("disabled") {
                w::dropdown_disabled::<&str, Message>(sel.checked_sub(1).map(|i| &FRUITS[i])).into()
            } else {
                w::dropdown(
                    &FRUITS,
                    sel.checked_sub(1).map(|i| FRUITS[i]),
                    move |s: &'static str| {
                        let i = FRUITS.iter().position(|f| *f == s).map_or(0, |i| i + 1);
                        Message::Play(d, PMsg::Choice("selected", i))
                    },
                )
                .into()
            };
            el
        }
        Detail::ActionRow => {
            let mut r = w::ActionRow::new(p.text("title").to_string()).metrics(m);
            if p.bool("subtitle") {
                r = r.subtitle("A secondary line");
            }
            if p.bool("prefix") {
                r = r.prefix(cmp::avatar(40.0, p.text("title").to_string()));
            }
            if p.bool("suffix") {
                r = r.suffix(cmp::raised_button(lbl("Action")).on_press(Message::Noop));
            }
            if p.bool("chevron") {
                r = r.chevron(true).on_activate(Message::ShowToast("Opened"));
            }
            stage(Element::from(r))
        }
        Detail::TreeRow => {
            let selected = p.bool("selected");
            let mut r = w::TreeRow::new("Notifications")
                .metrics(m)
                .expanded(p.bool("expanded"))
                .on_toggle(on_bool(d, "expanded"))
                .selected(selected)
                .on_activate(Message::Play(d, PMsg::Bool("selected", !selected)));
            if p.bool("subtitle") {
                r = r.subtitle("Expand for details");
            }
            if p.bool("prefix") {
                r = r.prefix(Icon::Home.view(m.icon_size));
            }
            if p.bool("suffix") {
                r = r.suffix(cmp::Badge::count(p.number("count") as u32, 99));
            }
            // Untouched, the row keeps the toolkit's default (`[rows] expander-indent`).
            if p.changed("indent") {
                r = r.indent(p.number("indent"));
            }
            let count = p.number("count") as usize;
            let kind = p.choice("children");
            match kind {
                0 => {
                    for i in 0..count {
                        r = r.push(w::ActionRow::new(format!("Row {}", i + 1)).metrics(m));
                    }
                }
                1 => {
                    r = r
                        .push(w::SwitchRow::new("A switch row", true, |_| Message::Noop).metrics(m))
                        .push(
                            w::ActionRow::new("An action row")
                                .subtitle("With a subtitle")
                                .metrics(m),
                        )
                        .push(
                            iced::widget::container(ty::body("Any element can be a child"))
                                .padding(m.space_m),
                        );
                }
                _ => {
                    for i in 0..count {
                        r = r.push(
                            w::TreeRow::new(format!("Group {}", i + 1))
                                .metrics(m)
                                .expanded(i == 0)
                                .push(w::ActionRow::new("Nested").metrics(m)),
                        );
                    }
                }
            }
            Element::from(r)
        }
        Detail::ButtonRow => {
            let mut r = w::ButtonRow::new(p.text("title").to_string()).on_activate(Message::Noop);
            r = match p.choice("role") {
                1 => r.role(Role::Accent),
                2 => r.role(Role::Destructive),
                _ => r,
            };
            stage(Element::from(r))
        }
        Detail::Card => {
            let n = p.number("children") as usize;
            let pad = p.number("padding");
            let mut c = w::Card::new(ty::body("Child 1"))
                .padding(pad)
                .separators(p.bool("separators"));
            for i in 1..n {
                c = c.push(ty::body(format!("Child {}", i + 1)));
            }
            Element::from(c)
        }
        Detail::Clamp => {
            let max = p.number("max");
            let el = w::clamp(
                cmp::entry("Never wider than the maximum", "centered")
                    .on_input(|_| Message::Noop)
                    .metrics(m),
                max,
            );
            stage(el)
        }
        Detail::StatusPage => {
            let icon = [Icon::Search, Icon::Home, Icon::Settings][p.choice("icon")];
            let mut page = w::status_page::<Message>()
                .icon(icon.colored(64.0, g.build_theme().semantic.dim_solid()))
                .title(p.text("title").to_string())
                .description(p.text("description").to_string())
                .compact(true)
                .metrics(m);
            if p.bool("action") {
                page = page
                    .child(cmp::suggested_pill_button(lbl("Try Again")).on_press(Message::Noop));
            }
            container(page).height(Length::Fixed(280.0)).into()
        }
        Detail::WrapBox => {
            let tags: Vec<Element<'a>> = [
                "rust",
                "iced",
                "gnome",
                "adwaita",
                "libadwaita",
                "theming",
                "widgets",
                "rows",
                "cards",
                "menus",
                "popovers",
                "toasts",
            ]
            .into_iter()
            .take(p.number("count") as usize)
            .map(|t| cmp::tag(t, Role::Accent))
            .collect();
            stage(w::wrap_box(tags, p.number("spacing")))
        }
        Detail::Collapse => {
            let behavior = if p.choice("behavior") == 0 {
                iad::CollapseBehavior::Stacked
            } else {
                iad::CollapseBehavior::Flex
            };
            let r = w::SliderRow::new("Volume", p.number("value"), 0.0..=100.0, on_num(d, "value"))
                .subtitle("Narrow the window: the control wraps")
                .basis(260.0)
                .collapse(behavior)
                .metrics(m);
            stage(Element::from(r))
        }
        Detail::Tree => {
            let el = g.tree_demo.play(p, m);
            el.map(Message::Tree)
        }
        Detail::Container => {
            // The page is narrower than a Lg container, so the preview is drawn at 40%: the
            // parent, the maximum width and the padding are all scaled, the proportions are real.
            const SCALE: f32 = 0.4;
            let sizes = [
                lay::ContainerSize::Xs,
                lay::ContainerSize::Sm,
                lay::ContainerSize::Md,
                lay::ContainerSize::Lg,
                lay::ContainerSize::Xl,
            ];
            let size = sizes
                .get(p.choice("size"))
                .copied()
                .unwrap_or(lay::ContainerSize::Px(p.number("custom")));
            let (fluid, pad, parent) = (p.bool("fluid"), p.number("padding"), p.number("parent"));
            let content_px = if fluid {
                parent
            } else {
                parent.min(size.width())
            };
            let tint = |a: f32| {
                move |t: &iad::Theme| iced::widget::container::Style {
                    background: Some(iad::color::alpha(t.semantic.accent.bg, a).into()),
                    border: iced::Border::default().rounded(4),
                    ..Default::default()
                }
            };
            let content = container(ty::caption(format!(
                "{:.0} px",
                (content_px - 2.0 * pad).max(0.0)
            )))
            .center_x(Length::Fill)
            .padding(m.space_m)
            .style(tint(0.6));
            let inner = lay::Container::new(content)
                .size(lay::ContainerSize::Px(size.width() * SCALE))
                .fluid(fluid)
                .padding(pad * SCALE);
            let parent_box = container(inner)
                .width(Length::Fixed(parent * SCALE))
                .padding([m.space_s, 0.0])
                .style(|t: &iad::Theme| iced::widget::container::Style {
                    border: iced::Border::default()
                        .rounded(4)
                        .width(1)
                        .color(t.semantic.border),
                    ..Default::default()
                });
            let note = ty::caption(format!(
                "Parent {parent:.0} px; the container is {content_px:.0} px wide, the content {:.0} px. Drawn at 40%.",
                (content_px - 2.0 * pad).max(0.0)
            ));
            column![note, parent_box].spacing(m.space_s).into()
        }
        Detail::Divider => {
            let variant = [
                cmp::DividerVariant::Solid,
                cmp::DividerVariant::Dashed,
                cmp::DividerVariant::Dotted,
            ][p.choice("variant")];
            let tone = [
                cmp::DividerTone::Border,
                cmp::DividerTone::Card,
                cmp::DividerTone::Bar,
                cmp::DividerTone::Popover,
                cmp::DividerTone::Sidebar,
            ][p.choice("tone")];
            let position = [
                cmp::LabelPosition::Left,
                cmp::LabelPosition::Center,
                cmp::LabelPosition::Right,
            ][p.choice("position")];
            let vertical = p.choice("orientation") == 1;
            let label = p.text("label").to_string();
            let mut div = cmp::divider::<Message>()
                .variant(variant)
                .tone(tone)
                .size(p.number("size"))
                .spacing(p.number("spacing"));
            if vertical {
                div = div.vertical();
            } else if !label.is_empty() {
                div = div.label(ty::body(label.clone())).label_position(position);
            }
            let preview: Element<'a> = if vertical {
                container(
                    row![ty::body("Left"), Element::from(div), ty::body("Right")]
                        .spacing(m.space_s)
                        .align_y(iced::Alignment::Center),
                )
                .height(48)
                .into()
            } else {
                column![
                    ty::body("Above the divider"),
                    Element::from(div),
                    ty::body("Below the divider")
                ]
                .into()
            };
            preview
        }
        Detail::Splitter => {
            let el = g.widgets_demo.split_play(p, m);
            el.map(Message::Widgets)
        }
        Detail::Docked => {
            let side = [
                iad::layouts::anchor::Side::Bottom,
                iad::layouts::anchor::Side::Top,
                iad::layouts::anchor::Side::Left,
                iad::layouts::anchor::Side::Right,
            ][p.choice("side")];
            let size = [
                lay::DockSize::Auto,
                lay::DockSize::Fixed(120.0),
                lay::DockSize::Fraction(0.4),
                lay::DockSize::FullScreen,
            ][p.choice("size")];
            let length = [
                lay::DockSize::FullScreen,
                lay::DockSize::Fixed(320.0),
                lay::DockSize::Fraction(0.5),
            ][p.choice("length")];
            let align = [
                iad::layouts::anchor::Align::Start,
                iad::layouts::anchor::Align::Center,
                iad::layouts::anchor::Align::End,
            ][p.choice("align")];
            let mode = [lay::DockMode::Push, lay::DockMode::Overlay][p.choice("mode")];
            let scope = [lay::DockScope::Content, lay::DockScope::Window][p.choice("scope")];
            let stage = container(
                column![
                    ty::heading("Content"),
                    ty::body("What the surface is docked to. An overlay dock fades and disables it; a push dock makes room.")
                        .style(iad::styles::text::dim),
                    cmp::raised_button(ty::label("A button")).on_press(Message::Noop),
                ]
                .spacing(m.space_s),
            )
            .padding(m.space_l)
            .width(Length::Fill)
            .height(Length::Fill);
            let surface = container(
                column![
                    ty::title_3("Docked"),
                    ty::body("A footer, a banner, a drawer or a sheet.")
                        .style(iad::styles::text::dim),
                ]
                .spacing(m.space_s),
            )
            .padding(m.space_l)
            .width(Length::Fill);
            let effect = |k: &str| {
                [
                    lay::Effect::Slide,
                    lay::Effect::Fade,
                    lay::Effect::None,
                ][p.choice(k)]
            };
            let duration = std::time::Duration::from_millis(p.number("duration") as u64);
            // Always given and always built, so closing it plays the exit: once it has left,
            // an overlay draws nothing and lets everything through.
            let mut el = lay::Dock::new(stage, surface)
                .open(p.bool("open"))
                .closing(true)
                // The preview's own fill is what shows behind the surface.
                .backdrop(|t| t.semantic.view.bg)
                .motion(effect("enter"), effect("exit"))
                .duration(duration)
                .starting_value(p.number("starting") / 100.0)
                .ending_value(p.number("ending") / 100.0)
                .side(side)
                .size(size)
                .length(length)
                .align(align)
                .mode(mode)
                .scope(scope)
                .margin(p.number("margin"))
                .border(p.bool("border"))
                .scrim(p.bool("scrim"))
                .on_outside(Message::Noop);
            match p.choice("easing") {
                1 => {
                    el = el.spec(iad::layouts::animation::Spec::Timed(
                        duration,
                        iad::layouts::animation::Easing::EaseInOutCubic,
                    ));
                }
                2 => {
                    el = el.spec(iad::layouts::animation::Spec::Timed(
                        duration,
                        iad::layouts::animation::Easing::Linear,
                    ));
                }
                _ => {}
            }
            let preview = container(Element::from(el))
                .width(Length::Fill)
                .height(Length::Fixed(260.0))
                .style(|t: &iad::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(t.semantic.view.bg)),
                    border: iced::Border::default().rounded(t.metrics.radius_m),
                    ..Default::default()
                });
            Element::from(preview)
        }
        Detail::NumberInput => {
            let el = g.widgets_demo.number_play(p, m);
            el.map(Message::Widgets)
        }
        Detail::ColorPicker => {
            let el = g.widgets_demo.color_play(p, m, g.surface_radius());
            el.map(Message::Widgets)
        }
        Detail::Timeline => {
            let n = p.number("count") as usize;
            let variant = [
                cmp::DividerVariant::Solid,
                cmp::DividerVariant::Dashed,
                cmp::DividerVariant::Dotted,
            ][p.choice("variant")];
            let titles = [
                "Created", "Assigned", "Reviewed", "Approved", "Merged", "Released",
            ];
            let mut t = w::Timeline::new((p.number("active") as usize).min(n - 1))
                .bullet_size(p.number("bullet"))
                .line_width(p.number("line_width"))
                .reverse_active(p.bool("reverse"))
                .align(if p.choice("align") == 1 {
                    w::TimelineAlign::Right
                } else {
                    w::TimelineAlign::Left
                })
                .metrics(m);
            for (i, title) in titles.iter().enumerate().take(n) {
                let mut item = w::TimelineItem::new(*title)
                    .body(
                        ty::caption(format!("{} min ago", (n - i) * 12))
                            .style(iad::styles::text::dim),
                    )
                    .line(variant);
                if p.bool("loading") && i + 1 == n {
                    item = item.loading(true);
                }
                if p.bool("invalid") && i == 1 {
                    item = item.invalid(true);
                }
                t = t.item(item);
            }
            Element::from(t)
        }
        Detail::Spoiler => {
            if p.choice("mode") == 1 {
                let el: iad::Element<'a, crate::widgets_demo::Msg> = column![
                    ty::caption(g.widgets_demo.spoiler_caption()),
                    g.widgets_demo.spoiler_items(m)
                ]
                .spacing(m.space_s)
                .into();
                el.map(Message::Widgets)
            } else {
                let open = p.bool("expanded");
                let paragraph = "A spoiler shows the start of something long and keeps the rest out of the way. The control under it only appears when the content is taller than the maximum height, and the cut fades into the page. Press it to slide the rest open, and again to hide it. ".repeat(3);
                let fixed = p.choice("width") == 1;
                let s = w::Spoiler::new(ty::body(paragraph))
                    .width(if fixed {
                        Length::Fixed(240.0)
                    } else {
                        Length::Fill
                    })
                    .max_height(p.number("max_height"))
                    .expanded(open)
                    .preview(p.bool("preview"))
                    .duration(std::time::Duration::from_millis(p.number("duration") as u64))
                    .on_toggle(on_bool(d, "expanded"))
                    .metrics(m);
                Element::from(s)
            }
        }
        Detail::NoteCard => {
            let tint = [
                None,
                Some(iced::Color::from_rgb(0.9, 0.2, 0.2)),
                Some(iced::Color::from_rgb(0.2, 0.7, 0.3)),
                Some(iced::Color::from_rgb(0.2, 0.4, 0.9)),
            ][p.choice("tint")];
            let body = "Review spirit and code organisation of the merged branch; no functional changes. Findings so far: split the config module, refresh its stale docs, move the window accessors out, audit float literals, dedupe the theme folders. ".repeat(2);
            let content = column![
                ty::body("Review the merge"),
                w::Spoiler::new(ty::body(body))
                    .max_height(p.number("max_height"))
                    .preview(true)
                    .metrics(m),
            ]
            .spacing(m.space_s);
            Element::from(
                w::NoteCard::new(content)
                    .tint(tint)
                    .selected(p.bool("selected"))
                    .width(Length::Fixed(260.0))
                    .on_press(Message::Noop),
            )
        }
        Detail::Tabs => {
            let names = [
                ("Gallery", Icon::Home),
                ("Messages", Icon::Search),
                ("Settings", Icon::Settings),
            ];
            let variant = [
                w::TabsVariant::Default,
                w::TabsVariant::Outline,
                w::TabsVariant::Pills,
                w::TabsVariant::Floating,
            ][p.choice("variant")];
            let mut tabs = w::Tabs::new(Some(p.choice("selected")), on_pick(d, "selected"))
                .variant(variant)
                .grow(p.bool("grow"))
                .on_close(move |k: usize| {
                    // Closing a tab turns its control off: the app removes the tab.
                    let key = if k == 1 {
                        "show_messages"
                    } else {
                        "show_settings"
                    };
                    Message::Play(d, PMsg::Bool(key, false))
                })
                .metrics(m);
            for (i, (name, icon)) in names.into_iter().enumerate() {
                if (i == 1 && !p.bool("show_messages")) || (i == 2 && !p.bool("show_settings")) {
                    continue;
                }
                // The first tab is pinned: it has no close button.
                let mut t = w::TabItem::new(i, name)
                    .disabled(i == 2 && p.bool("disabled"))
                    .closable(p.bool("closable") && i != 0);
                if p.bool("icons") {
                    t = t.icon(icon);
                }
                if p.bool("panels") {
                    t = t.panel(move || ty::body(format!("{name} panel")).into());
                }
                tabs = tabs.tab(t);
            }
            Element::from(tabs)
        }
        Detail::ViewSwitcher => {
            let items = vec![
                cmp::ToggleItem::new(0usize, "Files").icon(Icon::Home),
                cmp::ToggleItem::new(1, "Search").icon(Icon::Search),
                cmp::ToggleItem::new(2, "Settings").icon(Icon::Settings),
            ];
            let el = Element::from(w::view_switcher(
                items,
                Some(p.choice("selected")),
                on_pick(d, "selected"),
            ));
            stage(el)
        }
        Detail::PathBar => {
            let el = w::path_bar(
                vec![
                    "Home".into(),
                    "Projects".into(),
                    "iced-adaptive-design".into(),
                ],
                p.choice("current"),
                on_pick(d, "current"),
            );
            stage(el)
        }
        Detail::ShortcutLabel => {
            stage(row![ty::body("Action"), w::shortcut_label(p.text("keys"))].spacing(m.space_m))
        }
        Detail::Banner => {
            let mut b = w::banner(p.text("title").to_string()).revealed(p.bool("revealed"));
            if p.bool("button") {
                b = b.button("Retry", Message::Play(d, PMsg::Bool("revealed", false)));
            }
            Element::from(b)
        }
        Detail::Progress => {
            let v = p.number("value");
            let bar = cmp::progress_bar(0.0..=100.0, v);
            let bar = match p.choice("role") {
                1 => bar.style(style::progress_bar::success),
                2 => bar.style(style::progress_bar::warning),
                3 => bar.style(style::progress_bar::error),
                _ => bar,
            };
            stage(Element::from(bar))
        }
        Detail::ClipWindow => {
            use iced::widget::{container as ct, row};
            let height = p.number("height");
            let anchor = if p.choice("anchor") == 0 {
                lay::ClipAnchor::Top
            } else {
                lay::ClipAnchor::Bottom
            };
            let content = || -> Element<'a> {
                column![
                    cmp::avatar(64.0, "Queue").shape(cmp::AvatarShape::Rounded),
                    ty::heading("Up next"),
                    ty::body("The content keeps its size"),
                    ty::caption("and slides into view"),
                ]
                .spacing(m.space_xs)
                .align_x(iced::Alignment::Center)
                .width(Length::Fill)
                .into()
            };
            let clipped = ct(Element::from(
                lay::ClipWindow::new(content())
                    .height(height)
                    .anchor(anchor),
            ))
            .width(Length::Fill)
            .style(iad::styles::container::card_surface);
            let squeezed = ct(content())
                .height(Length::Fixed(height))
                .width(Length::Fill)
                .style(iad::styles::container::card_surface);
            let el: Element<'a> = if p.bool("compare") {
                row![
                    column![ty::caption("ClipWindow: translated"), clipped]
                        .spacing(m.space_xs)
                        .width(Length::Fill),
                    column![
                        ty::caption("A container of that height: reflowed"),
                        squeezed
                    ]
                    .spacing(m.space_xs)
                    .width(Length::Fill),
                ]
                .spacing(m.space_m)
                .into()
            } else {
                clipped.into()
            };
            el
        }
        Detail::IconToggle => {
            let icon = [
                Icon::Shuffle,
                Icon::Repeat,
                Icon::Heart,
                Icon::Star,
                Icon::Queue,
            ][p.choice("icon")];
            let mut b = cmp::IconToggleButton::new(icon, p.bool("active"))
                .size(p.number("size"))
                .disabled(p.bool("disabled"))
                .on_toggle(on_bool(d, "active"));
            if p.bool("tooltip") {
                b = b.tooltip(format!("{icon:?}"));
            }
            stage(Element::from(b))
        }
        Detail::Avatar => {
            let shape = [
                cmp::AvatarShape::Circle,
                cmp::AvatarShape::Rounded,
                cmp::AvatarShape::Square,
            ][p.choice("shape")];
            let mut a = cmp::avatar(p.number("size"), p.text("name").to_string())
                .show_initials(p.bool("initials"))
                .shape(shape);
            if let Some(i) = icon_source(p.choice("icon")) {
                a = a.icon(i);
            }
            stage(Element::from(a))
        }
        Detail::Tag => {
            let r = p.choice("role");
            stage(cmp::tag(p.text("label").to_string(), ROLE_OF[r]))
        }
        Detail::BadgeChip => {
            let role = ROLE_OF[p.choice("role")];
            let label = p.text("label").to_string();
            let el: Element<'a> = match p.choice("kind") {
                0 => cmp::Badge::new(label.clone())
                    .role(role)
                    .solid(p.bool("solid"))
                    .into(),
                1 => cmp::Badge::count(p.number("count") as u32, 99)
                    .role(role)
                    .solid(p.bool("solid"))
                    .into(),
                3 => {
                    let size = p.number("size");
                    let icon = icon_source(p.choice("icon"));
                    let content: Element<'a> = match &icon {
                        Some(i) => i.view(size * 0.55).into(),
                        None => ty::caption(label.clone()).into(),
                    };
                    let mut b = cmp::RoundBadge::new(content)
                        .role(role)
                        .solid(p.bool("solid"))
                        .size(size);
                    if p.choice("color") == 1 {
                        b = b.color(iced::Color::from_rgb8(0xc0, 0x4a, 0x9a));
                    }
                    b.into()
                }
                _ => {
                    let mut c = cmp::Chip::new(label.clone())
                        .role(role)
                        .selected(p.bool("selected"))
                        .disabled(p.bool("disabled"))
                        .on_press(Message::Noop)
                        .on_remove(Message::Noop)
                        .size(
                            [
                                cmp::ChipSize::Small,
                                cmp::ChipSize::Medium,
                                cmp::ChipSize::Large,
                            ][p.choice("chip_size")],
                        );
                    if let Some(i) = icon_source(p.choice("icon")) {
                        c = c.icon(i);
                    }
                    if p.choice("color") == 1 {
                        c = c.color(
                            [
                                iced::Color::from_rgb8(0xc0, 0x4a, 0x9a),
                                iced::Color::from_rgb8(0x3b, 0x4a, 0x7a),
                                iced::Color::from_rgb8(0xe5, 0xa5, 0x0a),
                            ][p.choice("tint")],
                        );
                        if p.choice("contrast") == 1 {
                            c = c.min_contrast(0.0);
                        }
                        if p.choice("text") == 1 {
                            c = c.text_color(iced::Color::from_rgb8(0x2a, 0x2a, 0x2a));
                        }
                        if p.bool("on_color") {
                            c = c.on_color(iced::Color::WHITE);
                        }
                    }
                    c.into()
                }
            };
            stage(container(el).padding(m.space_s))
        }
        _ => return None,
    })
}
