//! A playground, after Mantine's docs "configurator" demos: one live component and a panel of
//! controls for its properties, with the code that builds what is shown. A page uses one
//! wherever a single instance can express the component; it keeps separate static demos only
//! where a playground cannot (several states side by side, a different layout per size).
//!
//! ```ignore
//! let play = Playground::new(vec![
//!     Spec::bool("disabled", "Disabled", false),
//!     Spec::choice("size", "Size", &["sm", "md", "lg"], 1),
//!     Spec::number("lift", "Lift", 1.0..=1.1, 1.02),
//! ]);
//! // update:  Msg::Play(m) => self.play.update(m)
//! // view:    self.play.view(metrics, preview, code, Msg::Play)
//! ```

use iced_libadwaita as adw;

use adw::widget::{self as w, ToggleItem, typography as ty};
use adw::{Element, Metrics};
use iced::Length;
use iced::widget::{column, container, row};
use std::ops::RangeInclusive;

/// One control: what it edits and how.
#[derive(Debug, Clone)]
pub struct Spec {
    key: &'static str,
    label: &'static str,
    kind: Kind,
}

#[derive(Debug, Clone)]
enum Kind {
    Bool(bool),
    Choice(&'static [&'static str], usize),
    Number(RangeInclusive<f32>, f32),
}

impl Spec {
    pub fn bool(key: &'static str, label: &'static str, default: bool) -> Self {
        Self {
            key,
            label,
            kind: Kind::Bool(default),
        }
    }
    /// One of `options`, by index.
    pub fn choice(
        key: &'static str,
        label: &'static str,
        options: &'static [&'static str],
        default: usize,
    ) -> Self {
        Self {
            key,
            label,
            kind: Kind::Choice(options, default),
        }
    }
    pub fn number(
        key: &'static str,
        label: &'static str,
        range: RangeInclusive<f32>,
        default: f32,
    ) -> Self {
        Self {
            key,
            label,
            kind: Kind::Number(range, default),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Value {
    Bool(bool),
    Choice(usize),
    Number(f32),
}

/// What a control reports.
#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Bool(&'static str, bool),
    Choice(&'static str, usize),
    Number(&'static str, f32),
    Reset,
}

pub struct Playground {
    specs: Vec<Spec>,
    values: Vec<Value>,
}

fn default_of(s: &Spec) -> Value {
    match &s.kind {
        Kind::Bool(d) => Value::Bool(*d),
        Kind::Choice(_, d) => Value::Choice(*d),
        Kind::Number(_, d) => Value::Number(*d),
    }
}

impl Playground {
    pub fn new(specs: Vec<Spec>) -> Self {
        let values = specs.iter().map(default_of).collect();
        Self { specs, values }
    }

    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Reset => self.values = self.specs.iter().map(default_of).collect(),
            Msg::Bool(k, v) => self.set(k, Value::Bool(v)),
            Msg::Choice(k, v) => self.set(k, Value::Choice(v)),
            Msg::Number(k, v) => self.set(k, Value::Number(v)),
        }
    }

    fn set(&mut self, key: &str, v: Value) {
        if let Some(i) = self.specs.iter().position(|s| s.key == key) {
            self.values[i] = v;
        }
    }

    fn get(&self, key: &str) -> Value {
        let i = self
            .specs
            .iter()
            .position(|s| s.key == key)
            .unwrap_or_else(|| panic!("no playground control `{key}`"));
        self.values[i]
    }

    pub fn bool(&self, key: &str) -> bool {
        matches!(self.get(key), Value::Bool(true))
    }
    /// The chosen index.
    pub fn choice(&self, key: &str) -> usize {
        match self.get(key) {
            Value::Choice(i) => i,
            _ => 0,
        }
    }
    pub fn number(&self, key: &str) -> f32 {
        match self.get(key) {
            Value::Number(n) => n,
            _ => 0.0,
        }
    }
    /// Whether `key` is not at its default (to leave defaults out of the code).
    pub fn changed(&self, key: &str) -> bool {
        self.specs
            .iter()
            .position(|s| s.key == key)
            .is_some_and(|i| self.values[i] != default_of(&self.specs[i]))
    }

    /// The preview, the properties panel and the code, in one column. `wrap` turns a control's
    /// [`Msg`] into the page's message.
    pub fn view<'a, M: Clone + 'a>(
        &'a self,
        m: Metrics,
        preview: Element<'a, M>,
        code: String,
        wrap: fn(Msg) -> M,
    ) -> Element<'a, M> {
        let group = |t: &'static str| w::PreferencesGroup::<M>::new().title(t).metrics(m);
        let mut props = group("Properties");
        for (s, v) in self.specs.iter().zip(&self.values) {
            props = props.push(control(s, *v, wrap, m));
        }
        let reset = w::flat_button(ty::label("Reset")).on_press(wrap(Msg::Reset));
        column![
            group("Preview").push(w::card(
                container(preview).padding(m.space_m).width(Length::Fill)
            )),
            row![container(props).width(Length::Fill)].spacing(m.space_s),
            container(reset).align_right(Length::Fill),
            group("Code").push(w::card(
                container(ty::monospace(code).size(13))
                    .padding(m.space_m)
                    .width(Length::Fill)
            )),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}

fn control<'a, M: Clone + 'a>(
    s: &Spec,
    v: Value,
    wrap: fn(Msg) -> M,
    m: Metrics,
) -> Element<'a, M> {
    let key = s.key;
    match (&s.kind, v) {
        (Kind::Bool(_), Value::Bool(on)) => {
            w::SwitchRow::new(s.label, on, move |b| wrap(Msg::Bool(key, b)))
                .metrics(m)
                .into()
        }
        (Kind::Choice(options, _), Value::Choice(i)) => {
            let items = options
                .iter()
                .enumerate()
                .map(|(n, o)| ToggleItem::new(n, *o))
                .collect();
            w::ToggleGroupRow::new(s.label, items, Some(i), move |n| wrap(Msg::Choice(key, n)))
                .metrics(m)
                .into()
        }
        (Kind::Number(range, _), Value::Number(n)) => {
            w::SliderRow::new(s.label, n, range.clone(), move |n| {
                // Two decimals: a slider is not a ruler.
                wrap(Msg::Number(key, (n * 100.0).round() / 100.0))
            })
            .metrics(m)
            .into()
        }
        _ => unreachable!("a control keeps the kind of its value"),
    }
}
