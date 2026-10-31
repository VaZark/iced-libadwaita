//! `AdwBanner`.

use super::Element;
use crate::style::{button as sb, container as sc};
use crate::Metrics;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::Length;

pub struct Banner<M> {
    title: String,
    button_label: Option<String>,
    on_button: Option<M>,
    revealed: bool,
    text_color: Option<iced::Color>,
    metrics: Metrics,
}

pub fn banner<M>(title: impl Into<String>) -> Banner<M> {
    Banner::new(title)
}

impl<M> Banner<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            button_label: None,
            on_button: None,
            revealed: true,
            text_color: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn button(mut self, label: impl Into<String>, on_press: M) -> Self {
        self.button_label = Some(label.into());
        self.on_button = Some(on_press);
        self
    }
    /// Hard override for the text and button color; by default it is chosen by contrast with
    /// the banner fill (CSS `color-contrast()`).
    pub fn text_color(mut self, c: iced::Color) -> Self {
        self.text_color = Some(c);
        self
    }
    pub fn revealed(mut self, r: bool) -> Self {
        self.revealed = r;
        self
    }
}

impl<'a, M: Clone + 'a> From<Banner<M>> for Element<'a, M> {
    fn from(b: Banner<M>) -> Self {
        let m = b.metrics;
        let override_fg = b.text_color;
        let mut r = row![
            container(text(b.title)).width(Length::Fill).align_x(Horizontal::Center)
        ]
        .align_y(Vertical::Center)
        .spacing(m.space_m);
        if let (Some(l), Some(msg)) = (b.button_label, b.on_button) {
            r = r.push(
                button(text(l))
                    .padding(m.button_padding)
                    .style(move |t, s| {
                        let mut st = sb::flat(t, s);
                        let fg = override_fg.unwrap_or(t.semantic.accent.fg);
                        st.text_color = fg;
                        // A state layer of the text color: light on dark fills, dark on light.
                        st.background = Some(iced::Background::Color(crate::color::alpha(
                            fg,
                            match s {
                                button::Status::Hovered => 0.25,
                                button::Status::Pressed => 0.35,
                                _ => 0.15,
                            },
                        )));
                        st
                    })
                    .on_press(msg),
            );
        }
        let banner = container(r)
            .padding([m.space_s, m.space_m])
            .width(Length::Fill)
            .style(move |t: &crate::Theme| {
                let mut st = sc::banner(t);
                if let Some(c) = override_fg {
                    st.text_color = Some(c);
                }
                st
            });
        // Hidden by default; `revealed` slides it in/out (250ms, ADW_EASE).
        super::reveal(banner, b.revealed).into()
    }
}
