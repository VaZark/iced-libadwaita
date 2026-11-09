//! The per-app theme config: the layers, live. Edit the toolkit theme file and the app theme
//! file as text, hardcode a token in code, and see which layer each token comes from.

use iced_adaptive_design as iad;

use iad::theme_config::{Schema, Source, ThemeConfig, Value, VariableType};
use iad::widgets::PreferencesGroup;
use iad::{Element, Metrics};
use iad::{components as cmp, widgets as w};
use iced::widget::{column, container, row, text};
use iced::{Color, Length};

#[derive(Debug, Clone)]
pub enum Msg {
    Toolkit(iced::widget::text_editor::Action),
    App(iced::widget::text_editor::Action),
    Hardcode(bool),
    Reset,
    Variation(&'static str),
}

pub struct State {
    toolkit: iced::widget::text_editor::Content,
    app: iced::widget::text_editor::Content,
    hardcode: bool,
    variation: &'static str,
}

const TOOLKIT: &str = "[metrics]\nradius-l=6\ncard-padding=14\n";
const APP: &str = "[cover]\nsize=120px\n[colors]\naccent=#e01b24\n[metrics]\nradius-l=24\n[dark:cover]\nbackground=hsla(270,50%,30%,1)\n[high-contrast:cover]\nsize=160px\nbackground=#000000\n";

/// The tokens this demo "app" declares for its own objects.
fn schema() -> Schema {
    Schema::new()
        .variations(["high-contrast"])
        .color("cover", "background", "#9c4ad1", "Behind a missing cover")
        .variable(
            "cover",
            "size",
            VariableType::Length {
                min: Some(32.0),
                max: Some(200.0),
            },
            Some(Value::Length(88.0)),
            "Cover edge: 32–200px",
        )
        .variable(
            "brand",
            "badge",
            VariableType::String {
                values: vec!["preview".into(), "release".into()],
                accepts_random: false,
            },
            None,
            "Optional badge; no default",
        )
        .boolean("cover", "rounded", true, "Round the cover's corners")
        .text("brand", "name", "Weave", "Shown in the header")
}

impl Default for State {
    fn default() -> Self {
        Self {
            toolkit: iced::widget::text_editor::Content::with_text(TOOLKIT),
            app: iced::widget::text_editor::Content::with_text(APP),
            hardcode: false,
            variation: "light",
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Toolkit(a) => self.toolkit.perform(a),
            Msg::App(a) => self.app.perform(a),
            Msg::Hardcode(h) => self.hardcode = h,
            Msg::Variation(key) => self.variation = key,
            Msg::Reset => *self = Self::default(),
        }
    }

    fn config(&self) -> ThemeConfig {
        let mut b = ThemeConfig::builder("org.example.Demo")
            .schema(schema())
            .files(false)
            .toolkit_text(self.toolkit.text())
            .app_text(self.app.text())
            .variation(self.variation);
        if self.hardcode {
            b = b.hardcode("cover", "size", Value::Length(64.0));
        }
        b.build()
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let cfg = self.config();
        fn edit<'a>(
            m: Metrics,
            title: &'static str,
            desc: &'static str,
            content: &'a iced::widget::text_editor::Content,
            f: fn(iced::widget::text_editor::Action) -> Msg,
        ) -> PreferencesGroup<'a, Msg> {
            PreferencesGroup::<Msg>::new()
                .title(title)
                .description(desc)
                .metrics(m)
                .push(w::card(
                    cmp::text_view(content)
                        .max_lines(8)
                        .min_lines(3)
                        .on_action(f),
                ))
        }

        // The resolved tokens: everything the app declares, and the toolkit's that were set.
        let mut table = PreferencesGroup::<Msg>::new()
            .title("Resolved")
            .description(
                "Hardcoded, then the app theme, then the toolkit theme, then the declared default.",
            )
            .metrics(m);
        for (spec, value, source) in cfg.entries() {
            let app_declared = schema().get(&spec.section, &spec.key).is_some();
            if !app_declared && source == Source::Default {
                continue;
            }
            let shown: Element<'_, Msg> = match value {
                Value::Color(c) => {
                    let c = *c;
                    row![
                        container(text(""))
                            .width(Length::Fixed(16.0))
                            .height(Length::Fixed(16.0))
                            .style(move |_: &iad::Theme| container::Style {
                                background: Some(c.into()),
                                border: iced::Border::default().rounded(4),
                                ..Default::default()
                            }),
                        text(value.to_text()).size(12),
                    ]
                    .spacing(m.space_xs)
                    .into()
                }
                other => text(other.to_text()).size(12).into(),
            };
            table = table.push(
                w::ActionRow::new(format!("[{}] {}", spec.section, spec.key))
                    .subtitle(format!("{}: {}", source.label(), spec.doc))
                    .suffix(shown)
                    .metrics(m),
            );
        }

        // A sample object styled only by tokens.
        let size = cfg.number("cover", "size").unwrap_or(88.0);
        let bg = cfg.color("cover", "background").unwrap_or(Color::BLACK);
        let radius = if cfg.bool("cover", "rounded").unwrap_or(true) {
            cfg.number("metrics", "radius-l").unwrap_or(12.0)
        } else {
            0.0
        };
        let pad = cfg.number("metrics", "card-padding").unwrap_or(12.0);
        let sample = container(
            row![
                container(text(cfg.text("brand", "name").unwrap_or("").to_string()))
                    .center(Length::Fixed(size))
                    .style(move |_: &iad::Theme| container::Style {
                        background: Some(bg.into()),
                        text_color: Some(Color::WHITE),
                        border: iced::Border::default().rounded(radius),
                        ..Default::default()
                    }),
                column![
                    text(format!(
                        "cover size {size:.0} px, radius {radius:.0}, card padding {pad:.0}"
                    ))
                    .size(12),
                    text(format!(
                        "accent token: {}",
                        iad::color::to_hex(cfg.color("colors", "accent").unwrap_or(Color::BLACK))
                    ))
                    .size(12),
                ]
                .spacing(m.space_xs),
            ]
            .spacing(m.space_m)
            .align_y(iced::Alignment::Center),
        )
        .padding(pad);
        let mut notes = column![].spacing(m.space_xs);
        for warn in cfg.warnings() {
            notes = notes.push(text(format!("warning: {warn}")).size(12));
        }
        for (s, k) in cfg.unknown() {
            notes = notes.push(text(format!("unknown, ignored: [{s}] {k}")).size(12));
        }
        for (s, k) in cfg.hardcoded() {
            notes = notes.push(text(format!("hardcoded (discouraged): [{s}] {k}")).size(12));
        }

        column![
            PreferencesGroup::new().title("Typed variables and variations").metrics(m)
                .description("Size accepts 32–200px. Invalid app values use the toolkit value, then the fallback. Optional values without a fallback remain unset.")
                .push(w::SelectEntry::new("Variation", &["light", "dark", "high-contrast"], Some(self.variation), Msg::Variation).metrics(m)),
            edit(
                m,
                "Toolkit theme file",
                "~/.config/iced-adaptive-design/theme.conf: the toolkit's tokens and their defaults. Lowest of the three.",
                &self.toolkit,
                Msg::Toolkit,
            ),
            edit(
                m,
                "App theme file",
                "~/.config/<app-id>/theme.conf: the app's own tokens, and any toolkit token, for this app only. Delete it and the app writes a fresh one from its declared defaults.",
                &self.app,
                Msg::App,
            ),
            PreferencesGroup::<Msg>::new()
                .title("In code")
                .description("Highest, and extremely discouraged: nobody can change it without a rebuild, and every one is reported at startup.")
                .metrics(m)
                .push(w::SwitchRow::new("Hardcode [cover] size = 64px", self.hardcode, Msg::Hardcode).metrics(m)),
            table,
            PreferencesGroup::<Msg>::new()
                .title("An object styled by tokens")
                .metrics(m)
                .push(w::card(column![sample, notes].spacing(m.space_s))),
            row![cmp::flat_button(text("Reset")).on_press(Msg::Reset)],
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
