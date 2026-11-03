//! The stateful pieces of the Divider, Splitter, Color picker, Timeline and Spoiler pages (the
//! stateless ones are in `plays.rs`): a splitter's sizes, the picked color and the spoiler's
//! shown count. The playgrounds' controls reach them through [`State::sync`] when the shape of
//! a demo changes.

use iced_libadwaita as adw;

use crate::playground::Playground;
use adw::widget::{
    self as w, ColorFormat, ColorInput, ColorPicker, Hsva, Redistribute, Splitter, SplitterEvent,
    SplitterLimit, SplitterOrientation, SplitterPane, SplitterState, Spoiler, SpoilerEvent,
    SpoilerState, typography as ty,
};
use adw::{Element, Metrics};
use iced::widget::{column, container};
use iced::{Length, Task};

#[derive(Debug, Clone)]
pub enum Msg {
    Split(SplitterEvent),
    ToggleFirst,
    ResetSplit,
    Color(Hsva),
    ColorText(String),
    Spoiler(SpoilerEvent),
}

pub struct State {
    split: SplitterState,
    /// What the splitter was built for, to rebuild it when a structural control changes.
    split_shape: Vec<usize>,
    color: Hsva,
    color_text: String,
    spoiler: SpoilerState,
    spoiler_shape: (usize, usize),
}

const SWATCHES: [&str; 8] = [
    "#e01b24", "#ff7800", "#f5c211", "#33d17a", "#2190a4", "#3584e4", "#9141ac", "#5e5c64",
];

impl Default for State {
    fn default() -> Self {
        let color = Hsva::parse("#3584e4").unwrap_or(Hsva::BLACK);
        Self {
            split: SplitterState::new(vec![SplitterPane::flex(), SplitterPane::flex()]),
            split_shape: Vec::new(),
            color,
            color_text: color.format(ColorFormat::Hex),
            spoiler: SpoilerState::new(100, 3).step(10),
            spoiler_shape: (3, 10),
        }
    }
}

fn shape(p: &Playground) -> Vec<usize> {
    vec![
        p.choice("panes"),
        p.bool("fixed") as usize,
        p.bool("collapsible") as usize,
        p.choice("redistribute"),
        p.bool("limits") as usize,
    ]
}

impl State {
    /// Rebuilds what a playground's structural controls decide.
    pub fn sync(&mut self, split: Option<&Playground>, spoiler: Option<&Playground>) {
        if let Some(p) = split
            && self.split_shape != shape(p)
        {
            self.split = build_split(p);
            self.split_shape = shape(p);
        }
        if let Some(p) = spoiler {
            let want = (p.number("initial") as usize, [0, 5, 10][p.choice("step")]);
            if want != self.spoiler_shape {
                let mut s = SpoilerState::new(100, want.0);
                if want.1 > 0 {
                    s = s.step(want.1);
                }
                self.spoiler = s;
                self.spoiler_shape = want;
            }
        }
    }

    pub fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Split(e) => {
                self.split.apply(e);
            }
            Msg::ToggleFirst => {
                self.split.toggle(0);
            }
            Msg::ResetSplit => self.split_shape.clear(),
            Msg::Color(c) => {
                self.color = c;
                self.color_text = c.format(ColorFormat::Hex);
            }
            Msg::ColorText(t) => {
                if let Some(c) = Hsva::parse(&t) {
                    self.color = c;
                }
                self.color_text = t;
            }
            Msg::Spoiler(e) => {
                self.spoiler.apply(e);
            }
        }
        Task::none()
    }

    // -- Splitter

    pub fn split_play<'a>(&'a self, p: &Playground, m: Metrics) -> (Element<'a, Msg>, String) {
        let vertical = p.choice("orientation") == 1;
        let sizes = self.split.sizes().to_vec();
        let mut s = Splitter::new(&self.split)
            .orientation(if vertical {
                SplitterOrientation::Vertical
            } else {
                SplitterOrientation::Horizontal
            })
            .with_handle(p.bool("grip"))
            .line_size(p.number("line"))
            .on_event(Msg::Split);
        for i in 0..self.split.len() {
            let note = format!(
                "Pane {}{}",
                i + 1,
                if self.split.is_collapsed(i) {
                    " (collapsed)"
                } else {
                    ""
                }
            );
            let _ = &sizes;
            s = s.pane(pane_box(&note, i));
        }
        let frame = container(s)
            .width(Length::Fill)
            .height(260)
            .style(|t: &adw::Theme| iced::widget::container::Style {
                border: iced::Border {
                    color: t.semantic.border,
                    width: 1.0,
                    radius: t.metrics.radius_m.into(),
                },
                ..Default::default()
            });
        let buttons = iced::widget::row![
            w::raised_button(ty::label("Toggle the first pane")).on_press(Msg::ToggleFirst),
            w::flat_button(ty::label("Reset sizes")).on_press(Msg::ResetSplit),
        ]
        .spacing(m.space_s);
        let preview: Element<'a, Msg> = column![buttons, Element::from(frame)]
            .spacing(m.space_m)
            .into();

        let mut code = String::from("// state\nSplitterState::new(vec![\n");
        for line in pane_code(p) {
            code.push_str(&format!("    {line},\n"));
        }
        code.push_str("])");
        if p.choice("redistribute") == 1 {
            code.push_str("\n    .redistribute(Redistribute::Nearest)");
        }
        code.push_str("\n\n// view\nSplitter::new(&self.split)\n");
        if vertical {
            code.push_str("    .orientation(SplitterOrientation::Vertical)\n");
        }
        if !p.bool("grip") {
            code.push_str("    .with_handle(false)\n");
        }
        if p.changed("line") {
            code.push_str(&format!("    .line_size({:.0}.0)\n", p.number("line")));
        }
        for i in 0..self.split.len() {
            code.push_str(&format!("    .pane(pane_{})\n", i + 1));
        }
        code.push_str("    .on_event(Msg::Split)\n\n// update: self.split.apply(event);   // self.split.toggle(0) from code");
        (preview, code)
    }

    // -- Color picker

    pub fn color_play<'a>(&'a self, p: &Playground, m: Metrics) -> (Element<'a, Msg>, String) {
        let formats = ColorFormat::ALL;
        let format = formats[p.choice("format")];
        let as_input = p.choice("kind") == 1;
        let swatches = p.bool("swatches");
        let el: Element<'a, Msg> = if as_input {
            let mut i = ColorInput::new(self.color, &self.color_text, Msg::ColorText, Msg::Color)
                .alpha(p.bool("alpha"))
                .format(format)
                .metrics(m);
            if swatches {
                i = i.swatches(SWATCHES);
            }
            i.into()
        } else {
            let mut c = ColorPicker::new(self.color, Msg::Color)
                .alpha(p.bool("alpha"))
                .format(format)
                .readout(p.bool("readout"))
                .width(p.number("width"))
                .metrics(m);
            if swatches {
                c = c.swatches(SWATCHES);
            }
            c.into()
        };
        let mut code = String::from(if as_input {
            "ColorInput::new(self.color, &self.text, Msg::Text, Msg::Color)\n"
        } else {
            "ColorPicker::new(self.color, Msg::Color)\n"
        });
        let mut line = |s: String| {
            code.push_str("    ");
            code.push_str(&s);
            code.push('\n');
        };
        if p.bool("alpha") {
            line(".alpha(true)".into());
        }
        if p.choice("format") != 0 {
            line(format!(".format(ColorFormat::{format:?})"));
        }
        if swatches {
            line(".swatches([\"#e01b24\", \"#ff7800\", \"#33d17a\", \"#3584e4\"])".into());
        }
        if !as_input && !p.bool("readout") {
            line(".readout(false)".into());
        }
        if !as_input && p.changed("width") {
            line(format!(".width({:.0}.0)", p.number("width")));
        }
        code.push_str(&format!(
            "\n// the color: Hsva; Hsva::parse(\"#3584e4\"), color.format(ColorFormat::{format:?}) = {}",
            self.color.format(format)
        ));
        (el, code)
    }

    // -- Spoiler

    pub fn spoiler_items<'a>(&'a self, m: Metrics) -> Element<'a, Msg> {
        Spoiler::items(&self.spoiler, move |i| {
            w::ActionRow::new(format!("Item {}", i + 1))
                .subtitle("Built only while it is shown")
                .metrics(m)
                .into()
        })
        .on_event(Msg::Spoiler)
        .into()
    }

    pub fn spoiler_caption(&self) -> String {
        format!(
            "{} of {} items built",
            self.spoiler.shown(),
            self.spoiler.total()
        )
    }
}

fn pane_code(p: &Playground) -> Vec<String> {
    let n = [2, 3][p.choice("panes")];
    (0..n)
        .map(|i| {
            let mut s = if i == 0 && p.bool("fixed") {
                "SplitterPane::fixed(240.0)".to_string()
            } else {
                "SplitterPane::flex()".to_string()
            };
            if p.bool("limits") && i == 0 {
                s.push_str(".min(SplitterLimit::Px(120.0)).max(SplitterLimit::Fraction(0.6))");
            }
            if p.bool("collapsible") && i == 0 {
                s.push_str(".collapsible(true)");
            }
            s
        })
        .collect()
}

fn build_split(p: &Playground) -> SplitterState {
    let n = [2, 3][p.choice("panes")];
    let panes = (0..n)
        .map(|i| {
            let mut pane = if i == 0 && p.bool("fixed") {
                SplitterPane::fixed(240.0)
            } else {
                SplitterPane::flex()
            };
            if p.bool("limits") && i == 0 {
                pane = pane
                    .min(SplitterLimit::Px(120.0))
                    .max(SplitterLimit::Fraction(0.6));
            }
            if p.bool("collapsible") && i == 0 {
                pane = pane.collapsible(true);
            }
            pane
        })
        .collect();
    SplitterState::new(panes).redistribute(if p.choice("redistribute") == 1 {
        Redistribute::Nearest
    } else {
        Redistribute::Adjacent
    })
}

/// A tinted box naming its pane, so the panes read as separate areas.
fn pane_box<'a>(label: &str, i: usize) -> Element<'a, Msg> {
    let tint = [
        iced::Color::from_rgb(0.21, 0.52, 0.89),
        iced::Color::from_rgb(0.2, 0.82, 0.48),
        iced::Color::from_rgb(0.96, 0.76, 0.07),
    ][i % 3];
    container(ty::body(label.to_string()))
        .center(Length::Fill)
        .style(move |_: &adw::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color { a: 0.14, ..tint })),
            ..Default::default()
        })
        .into()
}
