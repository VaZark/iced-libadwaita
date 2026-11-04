//! A playground, after Mantine's docs "configurator" demos: one live component and a panel of
//! controls for its properties, with the code that builds what is shown. A page uses one
//! wherever a single instance can express the component; it keeps separate static demos only
//! where a playground cannot (several states side by side, a different layout per size).
//!
//! ```ignore
//! let play = Playground::new(vec![
//!     Spec::bool("disabled", "Disabled", false),
//!     Spec::choice("size", "Size", &["sm", "md", "lg"], 1),
//!     Spec::number("count", "Count", 1.0..=12.0, 3.0),   // a number input
//!     Spec::slider("value", "Value", 0.0..=100.0, 40.0),  // a slider, to scrub a value
//! ]);
//! // update:  Msg::Play(m) => self.play.update(m)
//! // view:    self.play.view(metrics, preview, code, Msg::Play)
//! ```

use iced_adaptive_design as iad;

use iad::components::{ToggleItem, typography as ty};
use iad::{Element, Metrics};
use iad::{components as cmp, widgets as w};
use iced::Length;
use iced::widget::{column, container};
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
    /// A number typed or stepped: integers, or fixed decimals when the range is fractional.
    Number(RangeInclusive<f32>, f32),
    /// A value to scrub with a slider (a progress, an opacity), where dragging is the point.
    Slider(RangeInclusive<f32>, f32),
    Text(&'static str),
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
    pub fn text(key: &'static str, label: &'static str, default: &'static str) -> Self {
        Self {
            key,
            label,
            kind: Kind::Text(default),
        }
    }
    /// A number in `range`, in a number input (steppable, typed). Use it for counts, sizes and
    /// spacings; whole numbers unless the range or the default is fractional.
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
    /// A number in `range` on a slider: for a value whose feel matters more than its digits.
    pub fn slider(
        key: &'static str,
        label: &'static str,
        range: RangeInclusive<f32>,
        default: f32,
    ) -> Self {
        Self {
            key,
            label,
            kind: Kind::Slider(range, default),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Bool(bool),
    Choice(usize),
    Number(f32),
    Text(String),
}

/// What a control reports.
#[derive(Debug, Clone)]
pub enum Msg {
    Bool(&'static str, bool),
    Choice(&'static str, usize),
    /// A slider moved.
    Number(&'static str, f32),
    /// What was typed (or stepped to) in a number input.
    NumberText(&'static str, String),
    Text(&'static str, String),
    Reset,
    /// The Preview / Code switch: `true` is Code.
    Tab(bool),
}

pub struct Playground {
    specs: Vec<Spec>,
    values: Vec<Value>,
    /// What each number input shows (empty for the other controls): kept as typed, so typing is
    /// not interrupted.
    texts: Vec<String>,
    /// The Code tab is showing (else Preview).
    code: bool,
}

/// What a number input shows at the default (empty for the other kinds).
fn default_text(s: &Spec) -> String {
    match &s.kind {
        Kind::Number(range, d) => {
            let rule = number_rule(range, *d);
            rule.format(from_f32(*d, rule.digits))
        }
        _ => String::new(),
    }
}

fn default_of(s: &Spec) -> Value {
    match &s.kind {
        Kind::Bool(d) => Value::Bool(*d),
        Kind::Choice(_, d) => Value::Choice(*d),
        Kind::Number(_, d) | Kind::Slider(_, d) => Value::Number(*d),
        Kind::Text(d) => Value::Text((*d).to_string()),
    }
}

impl Playground {
    pub fn new(specs: Vec<Spec>) -> Self {
        let values = specs.iter().map(default_of).collect();
        let texts = specs.iter().map(default_text).collect();
        Self {
            specs,
            values,
            texts,
            // GALLERY_CODE=1 starts every page on its Code tab (for screenshots).
            code: std::env::var_os("GALLERY_CODE").is_some(),
        }
    }

    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Tab(code) => self.code = code,
            Msg::Reset => {
                self.values = self.specs.iter().map(default_of).collect();
                self.texts = self.specs.iter().map(default_text).collect();
            }
            Msg::Bool(k, v) => self.set(k, Value::Bool(v)),
            Msg::Choice(k, v) => self.set(k, Value::Choice(v)),
            Msg::Number(k, v) => self.set(k, Value::Number(v)),
            Msg::NumberText(k, t) => self.type_in(k, t),
            Msg::Text(k, v) => self.set(k, Value::Text(v)),
        }
    }

    /// Text typed in a number input: it is kept as typed, and the value follows it when it is a
    /// number inside the range.
    fn type_in(&mut self, key: &str, text: String) {
        let Some(i) = self.specs.iter().position(|s| s.key == key) else {
            return;
        };
        if let Kind::Number(range, _) = &self.specs[i].kind
            && let Some(d) = number_rule(range, self.default_number(i)).parse(&text)
            && number_rule(range, self.default_number(i))
                .validate(&text)
                .is_ok()
        {
            self.values[i] = Value::Number(to_f32(d));
        }
        self.texts[i] = text;
    }

    fn default_number(&self, i: usize) -> f32 {
        match &self.specs[i].kind {
            Kind::Number(_, d) | Kind::Slider(_, d) => *d,
            _ => 0.0,
        }
    }

    fn set(&mut self, key: &str, v: Value) {
        if let Some(i) = self.specs.iter().position(|s| s.key == key) {
            self.values[i] = v;
        }
    }

    fn get(&self, key: &str) -> &Value {
        let i = self
            .specs
            .iter()
            .position(|s| s.key == key)
            .unwrap_or_else(|| panic!("no playground control `{key}`"));
        &self.values[i]
    }

    pub fn bool(&self, key: &str) -> bool {
        matches!(self.get(key), Value::Bool(true))
    }
    /// The chosen index.
    pub fn choice(&self, key: &str) -> usize {
        match self.get(key) {
            Value::Choice(i) => *i,
            _ => 0,
        }
    }
    pub fn text(&self, key: &str) -> &str {
        match self.get(key) {
            Value::Text(t) => t,
            _ => "",
        }
    }
    pub fn number(&self, key: &str) -> f32 {
        match self.get(key) {
            Value::Number(n) => *n,
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

    /// A Preview / Code switch over the component and its code, then the properties panel.
    /// With no controls (a page that only has a snippet) the preview is the page's own demo,
    /// unframed, and there is no panel. `wrap` turns a [`Msg`] into the page's message.
    pub fn view<'a, M: Clone + 'a>(
        &'a self,
        m: Metrics,
        preview: Element<'a, M>,
        code: String,
        wrap: impl Fn(Msg) -> M + Copy + 'a,
    ) -> Element<'a, M> {
        let tabs = tab_row(m, self.code, move |c| wrap(Msg::Tab(c)));
        let framed = !self.specs.is_empty();
        let stage: Element<'a, M> = if self.code {
            container(ty::monospace(code).size(13))
                .padding(m.space_m)
                .width(Length::Fill)
                .style(frame)
                .into()
        } else if framed {
            container(preview)
                .padding(m.space_l)
                .width(Length::Fill)
                .style(frame)
                .into()
        } else {
            preview
        };
        let mut page = column![tabs, stage].spacing(m.space_m);
        if framed {
            let mut props = w::PreferencesGroup::<M>::new()
                .title("Properties")
                .metrics(m);
            for ((s, v), text) in self.specs.iter().zip(&self.values).zip(&self.texts) {
                props = props.push(control(s, v, text, wrap, m));
            }
            let reset = cmp::flat_button(ty::label("Reset")).on_press(wrap(Msg::Reset));
            page = page
                .push(container(props).width(Length::Fill))
                .push(container(reset).align_right(Length::Fill));
        }
        page.into()
    }
}

/// The Preview / Code tabs: the library's `Tabs` in the floating variant, as in Mantine's docs.
fn tab_row<'a, M: Clone + 'a>(
    m: Metrics,
    code: bool,
    on_tab: impl Fn(bool) -> M + Copy + 'a,
) -> Element<'a, M> {
    w::Tabs::new(Some(code), on_tab)
        .variant(w::TabsVariant::Floating)
        .tab(w::TabItem::new(false, "Preview"))
        .tab(w::TabItem::new(true, "Code"))
        .metrics(m)
        .into()
}

/// The preview's frame: just a hairline border around the page's own background, so the
/// component sits as it would in an app. No fill, no card, no shadow.
fn frame(theme: &iad::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: iced::Border {
            color: theme.semantic.border,
            width: 1.0,
            radius: theme.metrics.radius_l.into(),
        },
        ..Default::default()
    }
}

/// A "Code" section: the snippet in a plain frame, like the playground's. Pages that cannot be a
/// playground show one too.
pub fn code_block<'a, M: 'a>(m: Metrics, code: String) -> Element<'a, M> {
    column![
        ty::heading("Code"),
        container(ty::monospace(code).size(13))
            .padding(m.space_m)
            .width(Length::Fill)
            .style(frame)
    ]
    .spacing(m.space_m)
    .into()
}

fn control<'a, M: Clone + 'a>(
    s: &Spec,
    v: &Value,
    text: &'a str,
    wrap: impl Fn(Msg) -> M + Copy + 'a,
    m: Metrics,
) -> Element<'a, M> {
    let key = s.key;
    match (&s.kind, v) {
        (Kind::Bool(_), Value::Bool(on)) => {
            w::SwitchRow::new(s.label, *on, move |b| wrap(Msg::Bool(key, b)))
                .metrics(m)
                .into()
        }
        (Kind::Choice(options, _), Value::Choice(i)) => {
            let items = options
                .iter()
                .enumerate()
                .map(|(n, o)| ToggleItem::new(n, *o))
                .collect();
            w::ToggleGroupRow::new(s.label, items, Some(*i), move |n| wrap(Msg::Choice(key, n)))
                .metrics(m)
                .into()
        }
        (Kind::Number(range, default), Value::Number(_)) => {
            let rule = number_rule(range, *default);
            // The input lives in the row's suffix; its text is the playground's.
            w::ActionRow::new(s.label)
                .suffix(
                    w::NumberInput::new(text, move |t| wrap(Msg::NumberText(key, t)))
                        .numeric(rule)
                        .width(120.0)
                        .metrics(m),
                )
                .metrics(m)
                .into()
        }
        (Kind::Slider(range, _), Value::Number(n)) => {
            w::SliderRow::new(s.label, *n, range.clone(), move |n| {
                // Two decimals: a slider is not a ruler.
                wrap(Msg::Number(key, (n * 100.0).round() / 100.0))
            })
            .metrics(m)
            .into()
        }
        (Kind::Text(_), Value::Text(t)) => w::EntryRow::new(s.label, t.clone())
            .on_input(move |t| wrap(Msg::Text(key, t)))
            .metrics(m)
            .into(),
        _ => unreachable!("a control keeps the kind of its value"),
    }
}

/// The number input's rule for a range: whole numbers, or two decimals when the range or the
/// default is fractional; stepping by one unit of the last digit (1 for whole numbers).
fn number_rule(range: &RangeInclusive<f32>, default: f32) -> iad::primitives::numeric::Numeric {
    let whole = |v: f32| (v - v.round()).abs() < 1e-6;
    let digits = if whole(*range.start()) && whole(*range.end()) && whole(default) {
        0
    } else {
        2
    };
    iad::primitives::numeric::Numeric::decimal(digits)
        .min(from_f32(*range.start(), digits))
        .max(from_f32(*range.end(), digits))
}

fn from_f32(v: f32, digits: u8) -> iad::primitives::numeric::Decimal {
    iad::primitives::numeric::Decimal::new(
        (f64::from(v) * 10f64.powi(i32::from(digits))).round() as i64,
        digits,
    )
}

fn to_f32(d: iad::primitives::numeric::Decimal) -> f32 {
    (d.units as f64 / 10f64.powi(i32::from(d.digits))) as f32
}
