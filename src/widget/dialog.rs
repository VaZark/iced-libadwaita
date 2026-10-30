//! `AdwDialog`, `AdwAlertDialog`, `AdwAboutDialog`.

use super::Element;
use crate::style::{button as sb, container as sc, text as st};
use crate::{Metrics, Role, Theme, Typography};
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, mouse_area, opaque, row, space, stack, text};
use iced::Length;

/// Presents `dialog` modally over `base` with a dimmed backdrop.
/// `on_backdrop` is sent when the backdrop is clicked.
pub fn dialog<'a, M: Clone + 'a>(
    base: impl Into<Element<'a, M>>,
    dialog: impl Into<Element<'a, M>>,
    on_backdrop: Option<M>,
) -> Element<'a, M> {
    let mut scrim = mouse_area(container(space()).width(Length::Fill).height(Length::Fill).style(sc::backdrop));
    if let Some(m) = on_backdrop {
        scrim = scrim.on_press(m);
    }
    stack![
        base.into(),
        opaque(scrim),
        container(opaque(dialog.into())).center(Length::Fill).padding(12)
    ]
    .into()
}

/// Appearance of an [`AlertDialog`] response (`AdwResponseAppearance`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseAppearance {
    #[default]
    Default,
    Suggested,
    Destructive,
}

/// `AdwAlertDialog`. Turn it into an element and present it with [`dialog`].
pub struct AlertDialog<'a, M> {
    heading: String,
    body: Option<String>,
    child: Option<Element<'a, M>>,
    responses: Vec<(String, ResponseAppearance, M)>,
    metrics: Metrics,
}

impl<'a, M> AlertDialog<'a, M> {
    pub fn new(heading: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            body: None,
            child: None,
            responses: Vec::new(),
            metrics: Metrics::default(),
        }
    }
    pub fn body(mut self, b: impl Into<String>) -> Self {
        self.body = Some(b.into());
        self
    }
    /// `extra-child`.
    pub fn extra_child(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.child = Some(e.into());
        self
    }
    /// `add_response` + `set_response_appearance`.
    pub fn response(
        mut self,
        label: impl Into<String>,
        appearance: ResponseAppearance,
        on_press: M,
    ) -> Self {
        self.responses.push((label.into(), appearance, on_press));
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

fn separator_v<'a, M: 'a>() -> Element<'a, M> {
    container(space().width(1)).height(Length::Fill).style(sc::card_separator).into()
}

fn separator_h<'a, M: 'a>() -> Element<'a, M> {
    container(space().height(1)).width(Length::Fill).style(sc::card_separator).into()
}

impl<'a, M: Clone + 'a> From<AlertDialog<'a, M>> for Element<'a, M> {
    fn from(a: AlertDialog<'a, M>) -> Self {
        let m = a.metrics;
        let t = Typography::default();
        let mut body = column![t.title_2(a.heading).align_x(Horizontal::Center)]
            .spacing(m.space_m)
            .align_x(Horizontal::Center)
            .width(Length::Fill);
        if let Some(b) = a.body {
            body = body.push(t.body(b).align_x(Horizontal::Center));
        }
        if let Some(c) = a.child {
            body = body.push(c);
        }

        let mut buttons = row![].height(Length::Fixed(m.button_min_height + 12.0));
        let n = a.responses.len();
        for (i, (label, appearance, msg)) in a.responses.into_iter().enumerate() {
            if i > 0 {
                buttons = buttons.push(separator_v());
            }
            let bold = appearance != ResponseAppearance::Default;
            let role = match appearance {
                ResponseAppearance::Default => Role::Neutral,
                ResponseAppearance::Suggested => Role::Accent,
                ResponseAppearance::Destructive => Role::Destructive,
            };
            let lbl = text(label)
                .font(iced::Font {
                    weight: if bold { iced::font::Weight::Bold } else { iced::font::Weight::Normal },
                    ..iced::Font::DEFAULT
                })
                .style(move |th: &Theme| iced::widget::text::Style {
                    color: Some(match role {
                        Role::Neutral => th.semantic.dialog.fg,
                        r => th.semantic.swatch(r).color,
                    }),
                });
            let radius = m.radius_l;
            buttons = buttons.push(
                button(container(lbl).center(Length::Fill))
                    .padding(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |th, s| {
                        let mut st = sb::flat(th, s);
                        st.border.radius = match (i, n) {
                            (_, 1) => iced::border::bottom(radius),
                            (0, _) => iced::border::bottom_left(radius),
                            (i, n) if i + 1 == n => iced::border::bottom_right(radius),
                            _ => 0.0.into(),
                        };
                        st
                    })
                    .on_press(msg),
            );
        }

        container(
            column![container(body).padding(m.dialog_padding), separator_h(), buttons]
        )
        .width(Length::Fill)
        .max_width(m.dialog_width)
        .clip(true)
        .style(sc::dialog)
        .into()
    }
}

/// `AdwAboutDialog` (content only; present it with [`dialog`]).
pub struct AboutDialog<'a, M> {
    application_name: String,
    developer_name: Option<String>,
    version: Option<String>,
    comments: Option<String>,
    website: Option<String>,
    license: Option<String>,
    icon: Option<Element<'a, M>>,
    on_close: Option<M>,
}

impl<'a, M> AboutDialog<'a, M> {
    pub fn new(application_name: impl Into<String>) -> Self {
        Self {
            application_name: application_name.into(),
            developer_name: None,
            version: None,
            comments: None,
            website: None,
            license: None,
            icon: None,
            on_close: None,
        }
    }
    pub fn developer_name(mut self, v: impl Into<String>) -> Self {
        self.developer_name = Some(v.into());
        self
    }
    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.version = Some(v.into());
        self
    }
    pub fn comments(mut self, v: impl Into<String>) -> Self {
        self.comments = Some(v.into());
        self
    }
    pub fn website(mut self, v: impl Into<String>) -> Self {
        self.website = Some(v.into());
        self
    }
    pub fn license(mut self, v: impl Into<String>) -> Self {
        self.license = Some(v.into());
        self
    }
    pub fn application_icon(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(e.into());
        self
    }
    pub fn on_close(mut self, m: M) -> Self {
        self.on_close = Some(m);
        self
    }
}

impl<'a, M: Clone + 'a> From<AboutDialog<'a, M>> for Element<'a, M> {
    fn from(a: AboutDialog<'a, M>) -> Self {
        let m = Metrics::default();
        let t = Typography::default();
        let mut col = column![].spacing(m.space_s).align_x(Horizontal::Center).width(Length::Fill);
        if let Some(i) = a.icon {
            col = col.push(i);
        }
        col = col.push(t.title_2(a.application_name));
        if let Some(d) = a.developer_name {
            col = col.push(t.body(d).style(st::dim));
        }
        if let Some(v) = a.version {
            col = col.push(
                container(t.caption(v))
                    .padding([2.0, 10.0])
                    .style(sc::tinted(Role::Accent)),
            );
        }
        if let Some(c) = a.comments {
            col = col.push(t.body(c).align_x(Horizontal::Center));
        }
        if let Some(w) = a.website {
            col = col.push(t.body(w).style(st::accent));
        }
        if let Some(l) = a.license {
            col = col.push(t.caption(l).style(st::dim));
        }
        let mut c = column![container(col).padding(m.dialog_padding)];
        if let Some(msg) = a.on_close {
            c = c.push(separator_h()).push(
                button(container(text("Close")).center(Length::Fill))
                    .padding(0)
                    .width(Length::Fill)
                    .height(Length::Fixed(m.button_min_height + 12.0))
                    .style(move |th, s| {
                        let mut st = sb::flat(th, s);
                        st.border.radius = iced::border::bottom(m.radius_l);
                        st
                    })
                    .on_press(msg),
            );
        }
        container(c)
            .width(Length::Fill)
            .max_width(m.dialog_width + 40.0)
            .clip(true)
            .style(sc::dialog)
            .into()
    }
}
