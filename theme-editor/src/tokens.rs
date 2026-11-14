//! The tokens page: groups of rows, one control per token chosen by its declared type.

use crate::{Addr, Editor, Msg, Variant};
use iad::components as cmp;
use iad::primitives::color_picker::Hsva;
use iad::primitives::numeric::Decimal;
use iad::theme_config::{Gradient, Spec, Value, VariableType};
use iad::widgets as w;
use iad::{Element, Icon};
use iced::widget::{column, container, row, space, text};
use iced::{Alignment, Color, Length};
use iced_adaptive_design as iad;

/// One edit of a gradient token.
#[derive(Debug, Clone)]
pub(crate) enum GradEdit {
    Angle(String),
    Pos(usize, String),
    Color(usize, String),
    /// Start a gradient where there is none.
    Start,
    Add,
    Remove(usize),
}

/// The first word of a key: how tokens are grouped.
fn prefix(key: &str) -> String {
    key.split('-').next().unwrap_or(key).to_string()
}

impl Editor {
    /// The tokens of `section`, grouped by the first word of their key (a word shared by at least
    /// two tokens is a group; the rest are "general").
    pub(crate) fn groups_of(&self, section: &str) -> Vec<(String, Vec<&Spec>)> {
        let in_section: Vec<&Spec> = self.specs.iter().filter(|s| s.section == section).collect();
        let count = |p: &str| in_section.iter().filter(|s| prefix(&s.key) == p).count();
        let mut groups: Vec<(String, Vec<&Spec>)> = Vec::new();
        for s in &in_section {
            let p = prefix(&s.key);
            let title = if count(&p) >= 2 { p } else { "general".to_string() };
            match groups.iter_mut().find(|(t, _)| *t == title) {
                Some((_, v)) => v.push(s),
                None => groups.push((title, vec![s])),
            }
        }
        groups
    }

    /// Whether the file sets the token in any variant.
    pub(crate) fn is_set_anywhere(&self, spec: &Spec) -> bool {
        ["", "light:", "dark:"]
            .iter()
            .any(|p| self.values.contains_key(&(format!("{p}{}", spec.section), spec.key.clone())))
    }

    /// How many tokens of `section` the file sets (in any variant).
    pub(crate) fn set_count(&self, section: &str) -> usize {
        self.specs
            .iter()
            .filter(|s| s.section == section && self.is_set_anywhere(s))
            .count()
    }

    /// Whether the search and the "modified only" switch let `spec` show.
    pub(crate) fn visible(&self, spec: &Spec) -> bool {
        if self.modified_only && !self.is_set_anywhere(spec) {
            return false;
        }
        let q = self.search.trim().to_lowercase();
        q.is_empty()
            || spec.key.to_lowercase().contains(&q)
            || spec.doc.to_lowercase().contains(&q)
            || spec.section.to_lowercase().contains(&q)
    }

    /// The keys of `section` and group `group` that the current variant sets.
    pub(crate) fn group_keys(&self, section: &str, group: &str) -> Vec<Addr> {
        self.groups_of(section)
            .into_iter()
            .filter(|(t, _)| t == group)
            .flat_map(|(_, specs)| specs)
            .map(|s| (self.section_of(section), s.key.clone()))
            .filter(|at| self.values.contains_key(at))
            .collect()
    }

    /// The page: the tokens of the current section, or of every section when searching.
    pub(crate) fn page(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let searching = !self.search.trim().is_empty();
        let sections: Vec<&String> = if searching {
            self.sections.iter().collect()
        } else {
            self.sections.iter().filter(|s| **s == self.section).collect()
        };
        let mut page = column![].spacing(m.group_spacing);
        if let Variant::Light | Variant::Dark = self.variant {
            page = page.push(
                text(format!(
                    "Editing the {} variant: values here win over the base on that color scheme.",
                    self.variant.label().to_lowercase()
                ))
                .size(m.space_m + 1.0)
                .style(iad::styles::text::dim),
            );
        }
        let mut any = false;
        for section in sections {
            for (title, specs) in self.groups_of(section) {
                let specs: Vec<&Spec> = specs.into_iter().filter(|s| self.visible(s)).collect();
                if specs.is_empty() {
                    continue;
                }
                any = true;
                page = page.push(self.group(section, &title, specs, searching, m));
            }
        }
        if !any {
            page = page.push(
                w::status_page()
                    .icon(Icon::Search.view(48.0))
                    .title("No tokens match")
                    .description(if self.modified_only {
                        "This file sets nothing that matches. Turn off “Modified only” to see every token."
                    } else {
                        "Try another word: the search looks at names, descriptions and sections."
                    }),
            );
        }
        page.into()
    }

    /// A group of rows with its reset and fold buttons.
    fn group<'a>(
        &'a self,
        section: &str,
        title: &str,
        specs: Vec<&'a Spec>,
        searching: bool,
        m: iad::Metrics,
    ) -> Element<'a, Msg> {
        let key = (section.to_string(), title.to_string());
        let folded = self.collapsed.contains(&key);
        let heading = if searching {
            format!("{section} · {title}")
        } else {
            title.to_string()
        };
        let set_here = self.group_keys(section, title).len();
        let mut buttons = row![].spacing(m.space_xs).align_y(Alignment::Center);
        if set_here > 0 {
            buttons = buttons.push(cmp::tooltip(
                cmp::flat_button(cmp::typography::label(format!("Reset {set_here}")))
                    .on_press(Msg::ResetGroup(section.to_string(), title.to_string())),
                "Remove the values this group sets in the current variant",
                iad::components::TooltipPosition::Bottom,
            ));
        }
        buttons = buttons.push(cmp::tooltip(
            cmp::icon_button(if folded { Icon::GoNext } else { Icon::GoDown })
                .on_press(Msg::ToggleGroup(key.0.clone(), key.1.clone())),
            if folded { "Show the tokens" } else { "Hide the tokens" },
            iad::components::TooltipPosition::Bottom,
        ));
        if folded {
            return row![
                text(heading).size(m.space_m + 2.0),
                space().width(Length::Fill),
                buttons
            ]
            .align_y(Alignment::Center)
            .into();
        }
        let mut g = w::preferences_group().title(heading).header_suffix(buttons).metrics(m);
        for spec in specs {
            g = g.push(self.token_row(spec, m));
        }
        g.into()
    }

    /// One token: a dot when the file sets it, its name, description, and a control for its type.
    fn token_row<'a>(&'a self, spec: &'a Spec, m: iad::Metrics) -> Element<'a, Msg> {
        let (raw, set_here) = self.shown(spec);
        let section = spec.section.clone();
        let key = spec.key.clone();
        let shown = self.texts.get(&(section.clone(), key.clone())).map_or("", String::as_str);
        let default = spec.default.as_ref().map(Value::to_text).unwrap_or_default();
        let control: Element<'a, Msg> = match &spec.ty {
            VariableType::Bool => {
                let (s, k) = (section.clone(), key.clone());
                cmp::switch(raw == "true")
                    .on_toggle(move |on| Msg::Toggle(s.clone(), k.clone(), on))
                    .into()
            }
            VariableType::String { values, .. } if !values.is_empty() => {
                let (s, k) = (section.clone(), key.clone());
                w::dropdown(values, Some(raw.clone()), move |v| Msg::Input(s.clone(), k.clone(), v)).into()
            }
            VariableType::Color => self.color_control(&section, &key, &raw, shown, m),
            VariableType::Gradient => self.gradient_control(&section, &key, &raw, m),
            VariableType::Integer { .. }
            | VariableType::Decimal { .. }
            | VariableType::Length { .. }
            | VariableType::Percentage { .. }
            | VariableType::Number => {
                let (s, k) = (section.clone(), key.clone());
                let (lo, hi, digits, step) = bounds(&spec.ty, &spec.section);
                let mut n = w::NumberInput::new(shown, move |v| Msg::Input(s.clone(), k.clone(), v))
                    .decimals(digits)
                    .step(step)
                    .width(150.0);
                if let Some(lo) = lo {
                    n = n.min(lo);
                }
                if let Some(hi) = hi {
                    n = n.max(hi);
                }
                let unit = match &spec.ty {
                    VariableType::Length { .. } => Some("px"),
                    VariableType::Percentage { .. } => Some("%"),
                    _ => None,
                };
                match unit {
                    Some(u) => row![n, text(u)].spacing(m.space_xs).align_y(Alignment::Center).into(),
                    None => n.into(),
                }
            }
            _ => {
                let (s, k) = (section.clone(), key.clone());
                cmp::entry("", shown)
                    .on_input(move |v| Msg::Input(s.clone(), k.clone(), v))
                    .width(Length::Fixed(320.0))
                    .into()
            }
        };
        let reset: Element<'a, Msg> = if set_here {
            cmp::tooltip(
                cmp::flat_button(cmp::typography::label("Reset"))
                    .on_press(Msg::Reset(section.clone(), key.clone())),
                "Back to the default",
                iad::components::TooltipPosition::Bottom,
            )
        } else {
            space().width(Length::Fixed(0.0)).into()
        };
        // A filled dot: this file sets the token. Nothing: it is the default.
        let dot: Element<'a, Msg> = container(space())
            .width(Length::Fixed(8.0))
            .height(Length::Fixed(8.0))
            .style(move |t: &iad::Theme| iced::widget::container::Style {
                background: set_here.then(|| t.semantic.window.fg.scale_alpha(0.75).into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: if set_here { 0.0 } else { 1.0 },
                    color: t.semantic.window.fg.scale_alpha(0.25),
                },
                ..Default::default()
            })
            .into();
        w::ActionRow::new(spec.key.as_str())
            .subtitle(format!("{}  (default {default})", spec.doc))
            .prefix(dot)
            .suffix(row![control, reset].spacing(m.space_s).align_y(Alignment::Center))
            .metrics(m)
            .into()
    }

    /// A swatch that opens a color picker, and the hex entry.
    fn color_control<'a>(
        &'a self,
        section: &str,
        key: &str,
        raw: &str,
        shown: &'a str,
        m: iad::Metrics,
    ) -> Element<'a, Msg> {
        let color = match VariableType::Color.parse(raw) {
            Some(Value::Color(c)) => c,
            _ => Color::TRANSPARENT,
        };
        let (s, k) = (section.to_string(), key.to_string());
        let picker = w::ColorPicker::new(Hsva::from_rgba(color.r, color.g, color.b, color.a), {
            let (s, k) = (s.clone(), k.clone());
            move |h: Hsva| {
                let (r, g, b, a) = h.to_rgba();
                Msg::Input(s.clone(), k.clone(), iad::color::to_hex(Color::from_rgba(r, g, b, a)))
            }
        })
        .alpha(true)
        .width(Length::Fixed(260.0));
        let panel = w::popover_spaced([Element::from(picker)], 420.0, m.space_s, m.space_m);
        row![
            w::Popover::new(swatch(color), panel),
            cmp::entry("#rrggbb", shown)
                .on_input(move |v| Msg::Input(s.clone(), k.clone(), v))
                .width(Length::Fixed(150.0)),
        ]
        .spacing(m.space_s)
        .align_y(Alignment::Center)
        .into()
    }

    /// The gradient editor: a preview strip, the angle, and the stops (color, position).
    fn gradient_control<'a>(&'a self, section: &str, key: &str, raw: &str, m: iad::Metrics) -> Element<'a, Msg> {
        let Some(g) = Gradient::parse(raw) else {
            return row![
                text("None: the flat color shows")
                    .size(m.space_m + 1.0)
                    .style(iad::styles::text::dim),
                cmp::flat_button(cmp::typography::label("Add a gradient")).on_press(Msg::Gradient(
                    section.to_string(),
                    key.to_string(),
                    GradEdit::Start,
                )),
            ]
            .spacing(m.space_s)
            .align_y(Alignment::Center)
            .into();
        };
        let at = |field: &str| format!("{section}.{key}.{field}");
        let edit = |e: GradEdit, s: &str, k: &str| Msg::Gradient(s.to_string(), k.to_string(), e);
        let (s, k) = (section.to_string(), key.to_string());

        let strip = container(space())
            .width(Length::Fixed(240.0))
            .height(Length::Fixed(22.0))
            .style({
                let g = g.clone();
                move |_: &iad::Theme| iced::widget::container::Style {
                    background: Some(g.background()),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    },
                    ..Default::default()
                }
            });
        let angle = row![
            text("Angle").size(m.space_m + 1.0),
            cmp::entry("90", self.gtexts.get(&at("angle")).cloned().unwrap_or_default())
                .on_input({
                    let (s, k) = (s.clone(), k.clone());
                    move |v| edit(GradEdit::Angle(v), &s, &k)
                })
                .width(Length::Fixed(80.0)),
            text("deg").size(m.space_m + 1.0),
            space().width(Length::Fill),
            cmp::tooltip(
                cmp::flat_button(cmp::typography::label("Add stop"))
                    .on_press_maybe((g.stops.len() < 8).then(|| edit(GradEdit::Add, &s, &k))),
                "Add a stop between the last two",
                iad::components::TooltipPosition::Bottom,
            ),
        ]
        .spacing(m.space_xs)
        .align_y(Alignment::Center);
        let mut stops = column![].spacing(m.space_xs);
        for (i, (_, color)) in g.stops.iter().enumerate() {
            let hex = iad::color::to_hex(*color);
            let picker = w::ColorPicker::new(Hsva::from_rgba(color.r, color.g, color.b, color.a), {
                let (s, k) = (s.clone(), k.clone());
                move |h: Hsva| {
                    let (r, gr, b, a) = h.to_rgba();
                    Msg::Gradient(
                        s.clone(),
                        k.clone(),
                        GradEdit::Color(i, iad::color::to_hex(Color::from_rgba(r, gr, b, a))),
                    )
                }
            })
            .alpha(true)
            .width(Length::Fixed(260.0));
            let panel = w::popover_spaced([Element::from(picker)], 420.0, m.space_s, m.space_m);
            stops = stops.push(
                row![
                    w::Popover::new(swatch(*color), panel),
                    text(hex).size(m.space_m + 1.0).width(Length::Fixed(80.0)),
                    cmp::entry("", self.gtexts.get(&at(&format!("pos{i}"))).cloned().unwrap_or_default())
                        .on_input({
                            let (s, k) = (s.clone(), k.clone());
                            move |v| edit(GradEdit::Pos(i, v), &s, &k)
                        })
                        .width(Length::Fixed(70.0)),
                    text("%").size(m.space_m + 1.0),
                    cmp::flat_button(cmp::typography::label("Remove"))
                        .on_press_maybe((g.stops.len() > 2).then(|| edit(GradEdit::Remove(i), &s, &k))),
                ]
                .spacing(m.space_xs)
                .align_y(Alignment::Center),
            );
        }
        column![strip, angle, stops].spacing(m.space_xs).into()
    }
}

/// A small square of `color`, the trigger of a color picker.
fn swatch<'a>(color: Color) -> Element<'a, Msg> {
    container(space())
        .width(Length::Fixed(26.0))
        .height(Length::Fixed(26.0))
        .style(move |t: &iad::Theme| iced::widget::container::Style {
            background: Some(color.into()),
            border: iced::Border {
                color: t.semantic.window.fg.scale_alpha(0.35),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// The limits, digits and step a number control of `ty` has. Tokens are px unless the section
/// says otherwise: typography ratios and the scale are small decimals.
pub(crate) fn bounds(ty: &VariableType, section: &str) -> (Option<Decimal>, Option<Decimal>, u8, Decimal) {
    let d = |s: &str| Decimal::parse(s);
    match ty {
        VariableType::Integer { min, max, step, .. } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            0,
            d(&step.to_string()).unwrap_or_else(|| Decimal::from(1)),
        ),
        VariableType::Decimal { no_decimals, min, max, step, .. } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            *no_decimals,
            step.and_then(|v| d(&v.to_string())).unwrap_or_else(|| Decimal::from(1)),
        ),
        VariableType::Length { min, max } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            0,
            Decimal::from(1),
        ),
        VariableType::Percentage { min, max, .. } => (
            min.and_then(|v| d(&v.to_string())),
            max.and_then(|v| d(&v.to_string())),
            0,
            Decimal::from(1),
        ),
        _ if section == "typography" => (d("0"), None, 4, d("0.05").unwrap()),
        _ => (d("0"), None, 2, d("0.5").unwrap()),
    }
}
