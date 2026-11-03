//! The playgrounds of the gallery's component pages (see `playground.rs`): for each page, the
//! controls (`specs`) and the live component built from their values (`build`), with the code
//! that builds it. A page that is not here shows its own demo: it needs more than a single
//! instance can express (async states, a wizard flow, app-wide theme settings, a layout that
//! depends on the container).

use super::*;
use crate::playground::{Msg as PMsg, Spec};
use iced::widget::{column, container, row};

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

/// The code shown under a preview: a constructor, then one line per property that is not at
/// its default.
struct Code(String);

impl Code {
    fn new(head: &str) -> Self {
        Self(format!("{head}\n"))
    }
    fn add(mut self, on: bool, line: &str) -> Self {
        if on {
            self.0.push_str("    ");
            self.0.push_str(line);
            self.0.push('\n');
        }
        self
    }
    fn done(self) -> String {
        self.0.trim_end().to_string()
    }
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
            S::choice("style", "Style", &["Default", "Suggested"], 0),
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
        Detail::EntryRow => vec![
            S::text("title", "Title", "Display name"),
            S::text("value", "Value", "Ada"),
            S::bool("password", "Password", false),
            S::bool("disabled", "Disabled", false),
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
            S::choice("side", "Side", &["Bottom", "Top", "Left", "Right"], 0),
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
            S::bool("open", "Open", true),
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
            S::number("initial", "Items shown first", 1.0..=10.0, 3.0),
            S::choice("step", "A press reveals", &["All", "5 more", "10 more"], 2),
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
        Detail::Avatar => vec![
            S::text("name", "Name", "Ada Lovelace"),
            S::number("size", "Size", 24.0..=96.0, 48.0),
            S::bool("initials", "Show initials", true),
        ],
        Detail::Tag => vec![
            S::text("label", "Label", "Accent"),
            S::choice("role", "Role", &ROLES, 0),
        ],
        Detail::BadgeChip => vec![
            S::choice("kind", "Kind", &["Badge", "Count badge", "Chip"], 0),
            S::text("label", "Label", "New"),
            S::choice("role", "Role", &ROLES, 0),
            S::bool("solid", "Solid", false),
            S::number("count", "Count", 0.0..=150.0, 7.0),
            S::bool("selected", "Selected (chip)", false),
            S::bool("disabled", "Disabled (chip)", false),
        ],
        _ => Vec::new(),
    }
}

/// The live component for `d` built from the playground's values, and its code.
pub fn build<'a>(
    g: &'a Gallery,
    d: Detail,
    p: &'a crate::playground::Playground,
    m: Metrics,
) -> Option<(Element<'a>, String)> {
    let lbl = |s: &str| -> Element<'a> { w::typography::label(s.to_string()).into() };
    let noop = Message::Noop;
    Some(match d {
        Detail::Button => {
            let style = p.choice("style");
            let label = lbl(p.text("label"));
            let press = (!p.bool("disabled")).then_some(Message::Noop);
            let button: Element<'a> = match style {
                0 => w::raised_button(label).on_press_maybe(press).into(),
                1 => w::flat_button(label).on_press_maybe(press).into(),
                2 => w::suggested_button(label).on_press_maybe(press).into(),
                3 => w::destructive_button(label).on_press_maybe(press).into(),
                4 => w::pill_button(label).on_press_maybe(press).into(),
                _ => w::suggested_pill_button(label).on_press_maybe(press).into(),
            };
            let ctor = [
                "raised_button",
                "flat_button",
                "suggested_button",
                "destructive_button",
                "pill_button",
                "suggested_pill_button",
            ][style];
            let code = Code::new(&format!("{ctor}(label(\"{}\"))", p.text("label")))
                .add(!p.bool("disabled"), ".on_press(Msg::Pressed)")
                .done();
            (button, code)
        }
        Detail::ButtonContent => {
            let icon = [
                None,
                Some(Icon::Home),
                Some(Icon::ListAdd),
                Some(Icon::Search),
            ][p.choice("icon")];
            let mut content = w::ButtonContent::new(p.text("label").to_string()).metrics(m);
            if let Some(i) = icon {
                content = content.icon(i);
            }
            let press = (!p.bool("disabled")).then_some(Message::Noop);
            let button: Element<'a> = if p.choice("style") == 1 {
                w::suggested_button(Element::from(content))
                    .on_press_maybe(press)
                    .into()
            } else {
                w::raised_button(Element::from(content))
                    .on_press_maybe(press)
                    .into()
            };
            let code = Code::new(&format!("ButtonContent::new(\"{}\")", p.text("label")))
                .add(
                    icon.is_some(),
                    &format!(".icon(Icon::{:?})", icon.unwrap_or(Icon::Home)),
                )
                .done();
            (button, code)
        }
        Detail::ToggleGroup => {
            let names = [
                ("One", Icon::Home),
                ("Two", Icon::Search),
                ("Three", Icon::Settings),
            ];
            let icons = p.bool("icons");
            let items: Vec<w::ToggleItem<'a, usize, Message>> = names
                .iter()
                .enumerate()
                .map(|(i, (n, ic))| {
                    let it = w::ToggleItem::new(i, *n);
                    if icons { it.icon(*ic) } else { it }
                })
                .collect();
            let mut tg = w::toggle_group(items, Some(p.choice("selected")), on_pick(d, "selected"));
            if p.bool("fill") {
                tg = tg.width(Length::Fill);
            }
            let code = Code::new("toggle_group(items, Some(selected), Msg::Select)")
                .add(
                    icons,
                    "// each item: ToggleItem::new(value, label).icon(..)",
                )
                .add(p.bool("fill"), ".width(Length::Fill)")
                .done();
            (Element::from(tg), code)
        }
        Detail::LinkButton => {
            let b = w::link_button(p.text("label").to_string());
            let b: Element<'a> = b
                .on_press_maybe((!p.bool("disabled")).then_some(noop))
                .into();
            let code = Code::new(&format!("link_button(\"{}\")", p.text("label")))
                .add(!p.bool("disabled"), ".on_press(Msg::Open)")
                .done();
            (b, code)
        }
        Detail::TextInput => {
            let state = [
                None,
                Some(w::EntryState::Error),
                Some(w::EntryState::Warning),
                Some(w::EntryState::Success),
            ][p.choice("state")];
            let value = p.text("value").to_string();
            let el: Element<'a> = if p.bool("password") {
                let mut e = w::password_entry("Password", &value).metrics(m);
                if !p.bool("disabled") {
                    e = e.on_input(on_text(d, "value"));
                }
                e.into()
            } else {
                let mut e = w::entry("Placeholder", &value).metrics(m);
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
            let code = Code::new(if p.bool("password") {
                "password_entry(\"Password\", &self.value)"
            } else {
                "entry(\"Placeholder\", &self.value)"
            })
            .add(!p.bool("disabled"), ".on_input(Msg::Value)")
            .add(
                state.is_some() && !p.bool("password"),
                &format!(
                    ".state(EntryState::{})",
                    ["", "Error", "Warning", "Success"][p.choice("state")]
                ),
            )
            .add(
                p.bool("fixed") && !p.bool("password"),
                ".width(Length::Fixed(240.0))",
            )
            .done();
            (el, code)
        }
        Detail::Switch => {
            let mut s = w::switch(p.bool("active"));
            if !p.bool("disabled") {
                s = s.on_toggle(on_bool(d, "active"));
            }
            let code = Code::new("switch(self.active)")
                .add(!p.bool("disabled"), ".on_toggle(Msg::Active)")
                .done();
            (s.into(), code)
        }
        Detail::CheckButton => {
            let mut c = w::check_button(p.text("label").to_string(), p.bool("active"));
            if !p.bool("disabled") {
                c = c.on_toggle(on_bool(d, "active"));
            }
            let code = Code::new(&format!(
                "check_button(\"{}\", self.active)",
                p.text("label")
            ))
            .add(!p.bool("disabled"), ".on_toggle(Msg::Active)")
            .done();
            (c.into(), code)
        }
        Detail::RadioButton => {
            let sel = Some(p.choice("selected"));
            let mk = |i: usize, name: &str| -> Element<'a> {
                if p.bool("disabled") {
                    w::radio_button(name.to_string(), i, sel, |_| Message::Noop).into()
                } else {
                    w::radio_button(name.to_string(), i, sel, on_pick(d, "selected")).into()
                }
            };
            let el =
                column![mk(0, "Radio A"), mk(1, "Radio B"), mk(2, "Radio C")].spacing(m.space_s);
            let code = Code::new("radio_button(\"Radio A\", 0, self.selected, Msg::Select)")
                .add(true, "// one per option, same `selected`")
                .done();
            (el.into(), code)
        }
        Detail::Scale => {
            let v = p.number("value");
            let mut col = column![w::scale(0.0..=100.0, v, on_num(d, "value"))].spacing(m.space_s);
            if p.bool("readout") {
                col = col.push(ty::dim_label(format!("value {v:.0}")));
            }
            let code = Code::new("scale(0.0..=100.0, self.value, Msg::Value)").done();
            (col.into(), code)
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
            let code = Code::new("dropdown(&FRUITS, self.selected, Msg::Pick)")
                .add(
                    p.bool("disabled"),
                    "// disabled: dropdown_disabled(self.selected)",
                )
                .done();
            (el, code)
        }
        Detail::ActionRow => {
            let mut r = w::ActionRow::new(p.text("title").to_string()).metrics(m);
            if p.bool("subtitle") {
                r = r.subtitle("A secondary line");
            }
            if p.bool("prefix") {
                r = r.prefix(w::avatar(40.0, p.text("title").to_string()));
            }
            if p.bool("suffix") {
                r = r.suffix(w::raised_button(lbl("Action")).on_press(Message::Noop));
            }
            if p.bool("chevron") {
                r = r.chevron(true).on_activate(Message::ShowToast("Opened"));
            }
            let code = Code::new(&format!("ActionRow::new(\"{}\")", p.text("title")))
                .add(p.bool("subtitle"), ".subtitle(\"A secondary line\")")
                .add(p.bool("prefix"), ".prefix(avatar(40.0, title))")
                .add(
                    p.bool("suffix"),
                    ".suffix(raised_button(label(\"Action\")).on_press(Msg::Action))",
                )
                .add(p.bool("chevron"), ".chevron(true).on_activate(Msg::Open)")
                .done();
            (stage(Element::from(r)), code)
        }
        Detail::EntryRow => {
            let mut r = w::EntryRow::new(p.text("title").to_string(), p.text("value").to_string())
                .metrics(m);
            if !p.bool("disabled") {
                r = r.on_input(on_text(d, "value"));
            }
            if p.bool("password") {
                r = r.password(true);
            }
            let code = Code::new(&format!(
                "EntryRow::new(\"{}\", &self.value)",
                p.text("title")
            ))
            .add(!p.bool("disabled"), ".on_input(Msg::Value)")
            .add(p.bool("password"), ".password(true)")
            .done();
            (stage(Element::from(r)), code)
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
                r = r.suffix(w::Badge::count(p.number("count") as u32, 99));
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
            let children = ["rows", "mixed elements", "nested tree rows"][kind];
            let code = Code::new("TreeRow::new(\"Notifications\")")
                .add(p.bool("subtitle"), ".subtitle(\"Expand for details\")")
                .add(p.bool("prefix"), ".prefix(Icon::Home.view(16.0))")
                .add(p.bool("suffix"), ".suffix(Badge::count(n, 99))")
                .add(
                    true,
                    ".expanded(self.open).on_toggle(Msg::Toggle)   // the arrow",
                )
                .add(
                    true,
                    ".selected(self.selected).on_activate(Msg::Select) // the title",
                )
                .add(
                    p.changed("indent"),
                    &format!(
                        ".indent({:.0}.0)   // omitted: the toolkit's [rows] expander-indent",
                        p.number("indent")
                    ),
                )
                .add(
                    true,
                    &format!(".push(..)   // {children}: rows, other TreeRows, or any element"),
                )
                .done();
            (Element::from(r), code)
        }
        Detail::ButtonRow => {
            let mut r = w::ButtonRow::new(p.text("title").to_string()).on_activate(Message::Noop);
            r = match p.choice("role") {
                1 => r.role(Role::Accent),
                2 => r.role(Role::Destructive),
                _ => r,
            };
            let code = Code::new(&format!("ButtonRow::new(\"{}\")", p.text("title")))
                .add(p.choice("role") == 1, ".role(Role::Accent)")
                .add(p.choice("role") == 2, ".role(Role::Destructive)")
                .add(true, ".on_activate(Msg::Activate)")
                .done();
            (stage(Element::from(r)), code)
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
            let code = Code::new("Card::new(child)")
                .add(n > 1, &format!(".push(..) // {} children", n))
                .add(p.changed("padding"), &format!(".padding({pad:.0})"))
                .add(!p.bool("separators"), ".separators(false)")
                .done();
            (Element::from(c), code)
        }
        Detail::Clamp => {
            let max = p.number("max");
            let el = w::clamp(
                w::entry("Never wider than the maximum", "centered")
                    .on_input(|_| Message::Noop)
                    .metrics(m),
                max,
            );
            let code = Code::new("clamp(entry, max)")
                .add(true, &format!("// max = {max:.0}"))
                .done();
            (stage(el), code)
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
                page =
                    page.child(w::suggested_pill_button(lbl("Try Again")).on_press(Message::Noop));
            }
            let code = Code::new("status_page()")
                .add(true, &format!(".icon(Icon::{icon:?}.colored(64.0, dim))"))
                .add(true, &format!(".title(\"{}\")", p.text("title")))
                .add(
                    true,
                    &format!(".description(\"{}\")", p.text("description")),
                )
                .add(
                    p.bool("action"),
                    ".child(suggested_pill_button(label(\"Try Again\")))",
                )
                .done();
            (container(page).height(Length::Fixed(280.0)).into(), code)
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
            .map(|t| w::tag(t, Role::Accent))
            .collect();
            let code = Code::new(&format!("wrap_box(tags, {:.0})", p.number("spacing"))).done();
            (stage(w::wrap_box(tags, p.number("spacing"))), code)
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
            let code = Code::new("SliderRow::new(\"Volume\", self.value, 0.0..=100.0, Msg::Value)")
                .add(true, ".basis(260.0)")
                .add(
                    true,
                    &format!(
                        ".collapse(CollapseBehavior::{})",
                        ["Stacked", "Flex"][p.choice("behavior")]
                    ),
                )
                .done();
            (stage(Element::from(r)), code)
        }
        Detail::Tree => {
            let (el, code) = g.tree_demo.play(p, m);
            (el.map(Message::Tree), code)
        }
        Detail::Divider => {
            let variant = [
                w::DividerVariant::Solid,
                w::DividerVariant::Dashed,
                w::DividerVariant::Dotted,
            ][p.choice("variant")];
            let tone = [
                w::DividerTone::Border,
                w::DividerTone::Card,
                w::DividerTone::Bar,
                w::DividerTone::Popover,
                w::DividerTone::Sidebar,
            ][p.choice("tone")];
            let position = [
                w::LabelPosition::Left,
                w::LabelPosition::Center,
                w::LabelPosition::Right,
            ][p.choice("position")];
            let vertical = p.choice("orientation") == 1;
            let label = p.text("label").to_string();
            let mut div = w::divider::<Message>()
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
            let code = Code::new("divider()")
                .add(vertical, ".vertical()")
                .add(
                    !vertical && !label.is_empty(),
                    &format!(".label(body(\"{label}\"))"),
                )
                .add(
                    !vertical && !label.is_empty() && p.choice("position") != 1,
                    &format!(".label_position(LabelPosition::{:?})", position),
                )
                .add(
                    p.choice("variant") > 0,
                    &format!(".variant(DividerVariant::{variant:?})"),
                )
                .add(
                    p.changed("size"),
                    &format!(".size({:.0}.0)", p.number("size")),
                )
                .add(
                    p.choice("tone") > 0,
                    &format!(".tone(DividerTone::{tone:?})"),
                )
                .add(
                    p.changed("spacing"),
                    &format!(".spacing({:.0}.0)", p.number("spacing")),
                )
                .done();
            (preview, code)
        }
        Detail::Splitter => {
            let (el, code) = g.widgets_demo.split_play(p, m);
            (el.map(Message::Widgets), code)
        }
        Detail::Docked => {
            let side =
                [w::Side::Bottom, w::Side::Top, w::Side::Left, w::Side::Right][p.choice("side")];
            let size = [
                w::DockSize::Auto,
                w::DockSize::Fixed(120.0),
                w::DockSize::Fraction(0.4),
                w::DockSize::FullScreen,
            ][p.choice("size")];
            let length = [
                w::DockSize::FullScreen,
                w::DockSize::Fixed(320.0),
                w::DockSize::Fraction(0.5),
            ][p.choice("length")];
            let align = [
                w::AnchorAlign::Start,
                w::AnchorAlign::Center,
                w::AnchorAlign::End,
            ][p.choice("align")];
            let mode = [w::DockMode::Push, w::DockMode::Overlay][p.choice("mode")];
            let scope = [w::DockScope::Content, w::DockScope::Window][p.choice("scope")];
            let stage = container(
                column![
                    ty::heading("Content"),
                    ty::body("What the surface is docked to. An overlay dock fades and disables it; a push dock makes room.")
                        .style(iad::style::text::dim),
                    w::raised_button(ty::label("A button")).on_press(Message::Noop),
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
                        .style(iad::style::text::dim),
                ]
                .spacing(m.space_s),
            )
            .padding(m.space_l)
            .width(Length::Fill);
            let el = w::Dock::new(stage, surface)
                .open(p.bool("open"))
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
            let preview = container(Element::from(el))
                .width(Length::Fill)
                .height(Length::Fixed(260.0))
                .style(|t: &iad::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(t.semantic.view.bg)),
                    border: iced::Border::default().rounded(t.metrics.radius_m),
                    ..Default::default()
                });
            let side_name = ["Bottom", "Top", "Left", "Right"][p.choice("side")];
            let size_code = [
                "",
                ".size(DockSize::Fixed(120.0))",
                ".size(DockSize::Fraction(0.4))",
                ".size(DockSize::FullScreen)   // the whole base, every border flush",
            ][p.choice("size")];
            let length_code = [
                "",
                ".length(DockSize::Fixed(320.0))",
                ".length(DockSize::Fraction(0.5))",
            ][p.choice("length")];
            let align_code = ["", ".align(Align::Center)", ".align(Align::End)"][p.choice("align")];
            let code = Code::new("Dock::new(content, surface)")
                .add(true, ".open(self.open)")
                .add(p.choice("side") != 0, &format!(".side(Side::{side_name})"))
                .add(p.choice("size") != 0, size_code)
                .add(p.choice("length") != 0, length_code)
                .add(p.choice("align") != 0, align_code)
                .add(
                    p.choice("mode") == 1,
                    ".mode(DockMode::Overlay)   // over the content: it is faded and disabled",
                )
                .add(
                    p.choice("scope") == 1,
                    ".scope(DockScope::Window)   // corners on the window's edge take its radius",
                )
                .add(!p.bool("border"), ".border(false)")
                .add(p.choice("mode") == 1 && !p.bool("scrim"), ".scrim(false)")
                .add(
                    p.choice("mode") == 1,
                    ".on_outside(Msg::Close)   // a press outside",
                )
                .done();
            (Element::from(preview), code)
        }
        Detail::NumberInput => {
            let (el, code) = g.widgets_demo.number_play(p, m);
            (el.map(Message::Widgets), code)
        }
        Detail::ColorPicker => {
            let (el, code) = g.widgets_demo.color_play(p, m);
            (el.map(Message::Widgets), code)
        }
        Detail::Timeline => {
            let n = p.number("count") as usize;
            let variant = [
                w::DividerVariant::Solid,
                w::DividerVariant::Dashed,
                w::DividerVariant::Dotted,
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
                            .style(iad::style::text::dim),
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
            let code = Code::new(&format!(
                "Timeline::new({})",
                (p.number("active") as usize).min(n - 1)
            ))
            .add(
                p.changed("bullet"),
                &format!(".bullet_size({:.0}.0)", p.number("bullet")),
            )
            .add(
                p.changed("line_width"),
                &format!(".line_width({:.0}.0)", p.number("line_width")),
            )
            .add(p.bool("reverse"), ".reverse_active(true)")
            .add(p.choice("align") == 1, ".align(TimelineAlign::Right)")
            .add(
                true,
                ".item(TimelineItem::new(\"Created\").body(caption(\"12 min ago\")))",
            )
            .add(
                p.choice("variant") > 0,
                &format!(
                    ".item(TimelineItem::new(\"Assigned\").line(DividerVariant::{variant:?}))"
                ),
            )
            .add(
                p.bool("invalid"),
                ".item(TimelineItem::new(\"Reviewed\").invalid(true))",
            )
            .add(
                p.bool("loading"),
                ".item(TimelineItem::new(\"Merged\").loading(true))",
            )
            .done();
            (Element::from(t), code)
        }
        Detail::Spoiler => {
            if p.choice("mode") == 1 {
                let el: iad::Element<'a, crate::widgets_demo::Msg> = column![
                    ty::caption(g.widgets_demo.spoiler_caption()),
                    g.widgets_demo.spoiler_items(m)
                ]
                .spacing(m.space_s)
                .into();
                let step = ["", ".step(5)", ".step(10)"][p.choice("step")];
                let code = format!(
                    "// state: 100 items, the first {} shown\nSpoilerState::new(100, {}){step}\n\n// view: only the shown items are built\nSpoiler::items(&self.shown, |i| row_for(&self.data[i]))\n    .on_event(Msg::Spoiler)\n\n// update: Msg::Spoiler(e) => {{ self.shown.apply(e); }}",
                    p.number("initial") as usize,
                    p.number("initial") as usize
                );
                (el.map(Message::Widgets), code)
            } else {
                let open = p.bool("expanded");
                let paragraph = "A spoiler shows the start of something long and keeps the rest out of the way. The control under it only appears when the content is taller than the maximum height, and the cut fades into the page. Press it to slide the rest open, and again to hide it. ".repeat(3);
                let s = w::Spoiler::new(ty::body(paragraph))
                    .max_height(p.number("max_height"))
                    .expanded(open)
                    .on_toggle(on_bool(d, "expanded"))
                    .metrics(m);
                let code = Code::new("Spoiler::new(long_text)")
                    .add(
                        p.changed("max_height"),
                        &format!(".max_height({:.0}.0)", p.number("max_height")),
                    )
                    .add(
                        true,
                        ".expanded(self.open).on_toggle(Msg::Open)   // controlled",
                    )
                    .done();
                (Element::from(s), code)
            }
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
            let code = Code::new("Tabs::new(Some(self.tab), Msg::Tab)")
                .add(
                    p.choice("variant") > 0,
                    &format!(
                        ".variant(TabsVariant::{})",
                        ["Default", "Outline", "Pills", "Floating"][p.choice("variant")]
                    ),
                )
                .add(p.bool("grow"), ".grow(true)")
                .add(
                    p.bool("closable"),
                    ".on_close(Msg::CloseTab)   // the app removes the tab",
                )
                .add(
                    true,
                    &format!(
                        ".tab(TabItem::new(0, \"Gallery\"){}{})",
                        if p.bool("icons") {
                            ".icon(Icon::Home)"
                        } else {
                            ""
                        },
                        if p.bool("panels") {
                            ".panel(|| gallery())"
                        } else {
                            ""
                        }
                    ),
                )
                .add(
                    p.bool("show_messages"),
                    &format!(
                        ".tab(TabItem::new(1, \"Messages\"){})",
                        if p.bool("closable") {
                            ".closable(true)"
                        } else {
                            ""
                        }
                    ),
                )
                .add(
                    p.bool("show_settings"),
                    &format!(
                        ".tab(TabItem::new(2, \"Settings\"){}{})",
                        if p.bool("disabled") {
                            ".disabled(true)"
                        } else {
                            ""
                        },
                        if p.bool("closable") {
                            ".closable(true)"
                        } else {
                            ""
                        }
                    ),
                )
                .done();
            (Element::from(tabs), code)
        }
        Detail::ViewSwitcher => {
            let items = vec![
                w::ToggleItem::new(0usize, "Files").icon(Icon::Home),
                w::ToggleItem::new(1, "Search").icon(Icon::Search),
                w::ToggleItem::new(2, "Settings").icon(Icon::Settings),
            ];
            let el = Element::from(w::view_switcher(
                items,
                Some(p.choice("selected")),
                on_pick(d, "selected"),
            ));
            (
                stage(el),
                Code::new("view_switcher(items, Some(self.selected), Msg::Select)").done(),
            )
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
            (
                stage(el),
                Code::new("path_bar(segments, self.current, Msg::Go)").done(),
            )
        }
        Detail::ShortcutLabel => {
            let code = Code::new(&format!("shortcut_label(\"{}\")", p.text("keys"))).done();
            (
                stage(
                    row![ty::body("Action"), w::shortcut_label(p.text("keys"))].spacing(m.space_m),
                ),
                code,
            )
        }
        Detail::Banner => {
            let mut b = w::banner(p.text("title").to_string()).revealed(p.bool("revealed"));
            if p.bool("button") {
                b = b.button("Retry", Message::Play(d, PMsg::Bool("revealed", false)));
            }
            let code = Code::new(&format!("banner(\"{}\")", p.text("title")))
                .add(p.bool("button"), ".button(\"Retry\", Msg::Retry)")
                .add(true, ".revealed(self.revealed)")
                .done();
            (Element::from(b), code)
        }
        Detail::Progress => {
            let v = p.number("value");
            let bar = w::progress_bar(0.0..=100.0, v);
            let bar = match p.choice("role") {
                1 => bar.style(style::progress_bar::success),
                2 => bar.style(style::progress_bar::warning),
                3 => bar.style(style::progress_bar::error),
                _ => bar,
            };
            let code = Code::new("progress_bar(0.0..=100.0, value)")
                .add(
                    p.choice("role") > 0,
                    &format!(
                        ".style(style::progress_bar::{})",
                        ["", "success", "warning", "error"][p.choice("role")]
                    ),
                )
                .done();
            (stage(Element::from(bar)), code)
        }
        Detail::Avatar => {
            let a = w::avatar(p.number("size"), p.text("name").to_string())
                .show_initials(p.bool("initials"));
            let code = Code::new(&format!(
                "avatar({:.0}, \"{}\")",
                p.number("size"),
                p.text("name")
            ))
            .add(!p.bool("initials"), ".show_initials(false)")
            .done();
            (stage(Element::from(a)), code)
        }
        Detail::Tag => {
            let r = p.choice("role");
            let code = Code::new(&format!(
                "tag(\"{}\", Role::{:?})",
                p.text("label"),
                ROLE_OF[r]
            ))
            .done();
            (stage(w::tag(p.text("label").to_string(), ROLE_OF[r])), code)
        }
        Detail::BadgeChip => {
            let role = ROLE_OF[p.choice("role")];
            let label = p.text("label").to_string();
            let (el, code): (Element<'a>, String) = match p.choice("kind") {
                0 => (
                    w::Badge::new(label.clone())
                        .role(role)
                        .solid(p.bool("solid"))
                        .into(),
                    Code::new(&format!("Badge::new(\"{label}\")"))
                        .add(true, &format!(".role(Role::{role:?})"))
                        .add(p.bool("solid"), ".solid(true)")
                        .done(),
                ),
                1 => (
                    w::Badge::count(p.number("count") as u32, 99)
                        .role(role)
                        .solid(p.bool("solid"))
                        .into(),
                    Code::new(&format!("Badge::count({}, 99)", p.number("count") as u32))
                        .add(true, &format!(".role(Role::{role:?})"))
                        .add(p.bool("solid"), ".solid(true)")
                        .done(),
                ),
                _ => (
                    w::Chip::new(label.clone())
                        .role(role)
                        .selected(p.bool("selected"))
                        .disabled(p.bool("disabled"))
                        .on_press(Message::Noop)
                        .on_remove(Message::Noop)
                        .into(),
                    Code::new(&format!("Chip::new(\"{label}\")"))
                        .add(true, &format!(".role(Role::{role:?})"))
                        .add(p.bool("selected"), ".selected(true)")
                        .add(p.bool("disabled"), ".disabled(true)")
                        .add(true, ".on_press(Msg::Press).on_remove(Msg::Remove)")
                        .done(),
                ),
            };
            (stage(container(el).padding(m.space_s)), code)
        }
        _ => return None,
    })
}

/// The code of a page that has no playground (its demo needs more than one instance, or it is
/// not a component with properties to play with).
pub fn snippet(d: Detail) -> Option<&'static str> {
    Some(match d {
        Detail::TableVirtual => {
            "// the app keeps a Viewport (from on_scroll) and passes only the window's rows\nlet window = self.viewport.window(TOTAL, ResponsiveTable::pitch(ROW), 4);   // primitives::virtualize\nlet rows: Vec<Person> = window.clone().map(person).collect();\n\nResponsiveTable::new(&rows)\n    .column(Column::new(\"Name\", |p| heading(&p.name).into()).min_width(160.0))\n    .virtualized(TOTAL, window.start, ROW)   // rows of a long list: spacers stand in for the rest\n    .on_scroll(Msg::Scroll)                  // Msg::Scroll(vp) => self.viewport = vp\n\n// a virtualized table is always a table: cards have no fixed height"
        }
        Detail::TableAsync => {
            "// rows fetched 50 at a time as they are scrolled to (primitives::loading)\nlet mut chunks = Chunks::new(TOTAL, 50);\n\n// Msg::Scroll(vp): ask for the chunks the window touches, once each\nfor chunk in chunks.request(vp.window(TOTAL, pitch, 4)) {\n    tasks.push(Task::perform(fetch(chunk), move |r| Msg::Fetched(chunk, r)));\n}\n// Msg::Fetched(chunk, r): cache it, then chunks.finish(chunk, r.map(|_| ()))\n\n// a row is a placeholder until its chunk arrives; a failed chunk is not asked for again\nmatch chunks.slot(row) {\n    Slot::Ready => ready_cells(row),\n    Slot::Loading => skeleton_cells(),\n    Slot::Failed(reason) => error_cells(reason, /* Retry: chunks.retry(chunk) */),\n}"
        }
        Detail::TreeAsync => {
            "// data: a branch with children still to come\nTreeNode::lazy(\"remote\", \"remote\")\n\n// update: opening it returns an effect; fetch, then merge\nif let Some(TreeEffect::Load(v)) = self.tree.apply(&mut self.data, event) {\n    return Task::perform(fetch(v.clone()), move |r| Msg::Loaded(v.clone(), r));\n}\nMsg::Loaded(v, r) => self.tree.finish_load(&mut self.data, &v, r)   // Err: shows the error and Retry"
        }
        Detail::TreeVirtualized => {
            "let total = self.tree.rows(&self.data).len();\nlet range = self.viewport.window(total, TREE_ROW_HEIGHT, 4);   // primitives::virtualize\n\nscrollable(Tree::new(&self.data, &self.tree).window(range).on_event(Msg::Tree))\n    .on_scroll(|vp| Msg::Scroll(vp.absolute_offset().y, vp.bounds().height))\n\n// rows have a fixed height; only the window is built, the rest is spacing"
        }
        Detail::TreeFiles => {
            "Tree::new(&self.files, &self.tree)\n    .lines(true)\n    .icons(|node, row| Some(if row.has_children { Icon::Folder } else { Icon::Document }))\n    .on_event(Msg::Tree)"
        }
        Detail::TreeDocs => {
            "Tree::new(&self.docs, &self.tree)\n    .draggable(DragMode::Row)\n    .highlight(&self.search)\n    .label(|p| {\n        let n = p.node.children.len();\n        if n == 0 { return p.text; }\n        row![p.text, Badge::count(n as u32, 99)].spacing(8).into()   // a count per branch\n    })\n    .on_event(Msg::Tree)\n\n// search: self.tree.set_expanded(tree::matching_ancestors(&self.docs, &query))"
        }
        Detail::Field => {
            "Field::new(\"Name\", entry(\"Ada\", &self.name).on_input(Msg::Name))\n    .required(true)\n    .validity(self.validity(&\"name\"))\n    .interaction(self.form.interaction(&spec, &\"name\"))\n    .layout(FieldLayout::Row) // or Stacked\n    .plain_label(true)"
        }
        Detail::Form => {
            "Form::new()\n    .step(Step::new(\"Profile\", move || {\n        Field::new(\"Name\", entry(\"Ada\", &self.name).on_input(Msg::Name))\n            .validity(self.validity(&\"name\"))\n            .into()\n    }))\n    .on_submit(Msg::Submit)\n    .on_reset(Msg::Reset)\n\n// update: self.form.submit(&spec, &|k| self.validity(k))"
        }
        Detail::Stepper => {
            "Stepper::new()\n    .active(state.active())\n    .orientation(StepperOrientation::Horizontal)\n    .on_step_click(Msg::Go)\n    .step(StepperStep::new(\"Account\").content(|| body(\"...\")))\n    .step(StepperStep::new(\"Profile\").description(\"Fruit and birthday\"))\n    .step(StepperStep::new(\"Review\").allow_click(true))\n    .completed(done)\n\n// state: StepperState::new(3), .forward() / .back() / .goto(i)"
        }
        Detail::Autocomplete => {
            "Autocomplete::new(&self.text, &self.lookup, Msg::Input)\n    .on_select(Msg::Pick)\n    .on_retry(Msg::Retry)\n    .item(|place: &Place| text(place.name).into())\n    .placeholder(\"Search cities\")\n    .search_icon(true)\n\n// the lookup: Lookup<Query, Place> debounces and drops stale responses"
        }
        Detail::Data => {
            "ResponsiveTable::new(&rows)\n    .column(Column::new(\"Name\", |r| text(r.name).into()).sortable(Col::Name))\n    .sort(&self.sort)                         // SortState: asc / desc / unsorted\n    .on_sort(Msg::Sort)\n    .footer(|| Pagination::new(&self.paging, Msg::Page).siblings(1).into())\n\n// InfiniteList for load-more: request ids discard stale pages"
        }
        Detail::TextView => {
            "text_view(&self.content)\n    .placeholder(\"Type here…\")\n    .on_action(Msg::Edit)   // omit for a read-only view\n\n// update: self.content.perform(action)"
        }
        Detail::Carousel => {
            "carousel(pages, self.position, Msg::Carousel)\n    .indicator(CarouselIndicator::Dots) // or Lines"
        }
        Detail::Indicator => {
            "Indicator::new(avatar(48.0, \"Ada\"), Badge::count(3, 99).role(Role::Error).solid(true))\n    .corner(Side::Bottom, Align::End)\n    .offset(4.0)\n    .show(unread > 0)"
        }
        Detail::ResponsiveTable => {
            "ResponsiveTable::<Person, Msg>::new(&PEOPLE)\n    .column(Column::new(\"Name\", |p| heading(p.name).into()).min_width(110.0))\n    .column(Column::new(\"Role\", |p| body(p.role).into()).min_width(90.0))\n    .mode(TableMode::Auto) // Auto (default), Table, Cards"
        }
        Detail::Accordion => {
            "Accordion::new(&self.open, Msg::Open)\n    .selection(Selection::Single)   // or Multiple\n    .item(AccordionItem::new(0u8, \"Shipping\", body).subtitle(\"2-4 days\"))\n    .item(AccordionItem::new(1u8, \"Returns\", body))"
        }
        Detail::CardButton => {
            "card_button(column![heading(\"Title\"), body(\"Any content\")])\n    .on_press(Msg::Open)"
        }
        Detail::DateInput => {
            "DateInput::new(&self.text, self.nav.date, Msg::Text)\n    .calendar(self.nav)           // CalendarView: Month / Year / Decade\n    .selected(self.pick)\n    .on_pick(Msg::Pick)\n    .on_nav(Msg::Nav)\n    .dialog(true)                 // a dialog over a faded window; .scrim(false) keeps it clear\n    .min(Date::new(2026, 11, 1))"
        }
        Detail::DateRange => {
            "DateRangeInput::new(&self.text, self.nav.date, Msg::Text)   // \"2026-11-02 \u{2013} 2026-11-09\"\n    .dialog(true)                 // a dialog over a faded window; .scrim(false) keeps it clear\n    .calendar(self.nav)\n    .selected(self.range)\n    .hover(self.hover)            // the preview of the range being picked"
        }
        Detail::RangeSlider => {
            "range_slider(0.0..=100.0, self.value, Msg::Range)\n\nRangeSliderRow::new(\"Volume\", self.value, 0.0..=100.0, Msg::Range)"
        }
        Detail::MultiSelect => {
            "MultiSelect::new(&FRUITS, &self.chosen, Msg::Toggle)\n    .placeholder(\"Pick fruits\")\n    .max_chips(2)\n    .width(Length::Fixed(260.0))"
        }
        Detail::AnchoredPopover => {
            "Popover::new(text(\"Open\"), popover([menu_item(\"First\", false, Msg::A)], 160.0))\n    .side(Side::Bottom)\n    .align(PopoverAlign::Start)\n    .offset(8.0)\n    .popover_width(140.0)\n    .bounds(PopoverBounds::Window)   // omitted: kept inside the content area it opened from\n\n// it flips to the other side and shifts to stay inside its bounds"
        }
        Detail::Surfaces => {
            "let sem = theme.semantic;\ncolor_swatch(\"card\", sem.card)\nstatus_swatch(\"accent\", sem.accent)\n\n// every surface and status color is a field of `Theme::semantic`"
        }
        Detail::Toast => {
            "toast_stack_from(content, &self.toasts, |toast| {\n    Toast::new(&toast.title).action(\"Undo\", Msg::Undo).status(Role::Success)\n})\n\n// self.toasts: notifications::Queue<_> (limit, replace, auto-dismiss from `now`)"
        }
        Detail::SwitchAndComboRows => {
            "SwitchRow::new(\"Enabled\", self.on, Msg::Toggle)\nComboRow::new(\"Font\", &FONTS, self.font, Msg::Font).disabled_when(|o| *o == \"Inter\")\nSpinRow::new(\"Text scale\", self.scale, 1.0..=10.0, 1.0, Msg::Scale)\nSliderRow::new(\"Volume\", self.volume, 0.0..=100.0, Msg::Volume)"
        }
        Detail::PreferencesGroup => {
            "preferences_group()\n    .title(\"Appearance\")\n    .description(\"How the app looks\")\n    .header_suffix(flat_button(label(\"Reset\")))\n    .push(SwitchRow::new(\"Dark Style\", self.dark, Msg::Dark))"
        }
        Detail::ThemeColors => {
            "Theme::new(Scheme::Dark)\n    .accent(AccentColor::Teal)\n    .with_semantic(|s| s.card.bg = hex(0x3d2c5a))\n\n// desktop config: [appearance] color-scheme=dark, accent=teal, density=compact"
        }
        Detail::ThemeShape => {
            "Theme::new(scheme)\n    .with_metrics(Metrics { radius_l: 12.0, radius_toast: 24.0, ..Metrics::default() })\n    .with_shadows(Shadows::default().with_scale(1.0).with_intensity(1.0))\n\n// no shadows: Shadows::none()"
        }
        Detail::ThemeMotion => {
            "// desktop config\n[motion]\nanimations=on        # off = every trigger jumps to its end\npage-push=none        # default = its own animation\n\n// in a widget\nTrigger::Reveal.spec(Spec::STYLE)   // resolves the user's choice"
        }
        Detail::ThemeWindow => {
            "// desktop config\n[menus]\npresentation=dropdown      # or full-page\n[navigation]\nmobile-sidebar=parent-page # or drawer\n\nprimary_menu(menu).presentation(MenuPresentation::Dropdown)"
        }
        Detail::MetaComponent => {
            "MetaComponent::new()\n    .selected(self.searching.then_some(Bar::Search))      // on-select trigger\n    .when_selected(Bar::Search, |_| search_entry())\n    .variant(Bar::Compact, Trigger::selected(Bar::Menu) | !wide, |_| menu())\n    .variant(Bar::Path, When::always(), |_| path_bar())   // fallback"
        }
        Detail::Resizable => {
            "Resizable::new(meta_component, self.frame)       // frame: iced::Size\n    .on_resize(Msg::Frame)\n    .constraints(Constraints::new(Size::new(220.0, 120.0), Size::new(900.0, 500.0))\n        .snap_to(breakpoints, 8.0))"
        }
        Detail::AlertDialog => {
            "AlertDialog::new(\"Discard Changes?\")\n    .body(\"Unsaved changes will be permanently lost.\")\n    .response(\"Cancel\", ResponseAppearance::Default, Msg::Close)\n    .response(\"Save\", ResponseAppearance::Suggested, Msg::Save)\n    .response(\"Discard\", ResponseAppearance::Destructive, Msg::Discard)"
        }
        Detail::AboutDialog => {
            "AboutDialog::new(\"Gallery\")\n    .developer_name(\"The authors\")\n    .version(\"0.1.0\")\n    .comments(\"Widget gallery.\")\n    .website(\"https://iced.rs\")\n    .license(\"MIT OR Apache-2.0\")"
        }
        Detail::ShortcutsDialog => {
            "shortcuts_dialog(vec![(\n    \"General\".into(),\n    vec![(\"Preferences\".into(), \"Ctrl+,\".into())],\n)], Msg::Close)"
        }
        Detail::PreferencesDialog => {
            "preferences_dialog(\n    \"Preferences\",\n    preferences_page().push(preferences_group().title(\"General\").push(row)),\n    Msg::Close,\n)"
        }
        Detail::AppShell => {
            "// gallery/src/appshell.rs: an app shell assembled from library parts\nAdaptive over Layouts              // width classes -> whole layouts; Close(old) then Open(new)\nSidebarList::new(items).follow(..) // the navbar and the drawer list\nheader_bar().title(\"Gallery\")\ndock_drawer(main, drawer, Side::Right, Dim::Fraction(0.4))"
        }
        _ => return None,
    })
}
