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
            metrics: Metrics::default(),
        }
    }
    pub fn button(mut self, label: impl Into<String>, on_press: M) -> Self {
        self.button_label = Some(label.into());
        self.on_button = Some(on_press);
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
        let mut r = row![
            container(text(b.title)).width(Length::Fill).align_x(Horizontal::Center)
        ]
        .align_y(Vertical::Center)
        .spacing(m.space_m);
        if let (Some(l), Some(msg)) = (b.button_label, b.on_button) {
            r = r.push(
                button(text(l))
                    .padding(m.button_padding)
                    .style(|t, s| {
                        let mut st = sb::flat(t, s);
                        st.text_color = t.semantic.accent.fg;
                        st.background = Some(iced::Background::Color(crate::color::white(
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
            .style(sc::banner);
        // Hidden by default; `revealed` slides it in/out (250ms, ADW_EASE).
        super::reveal(banner, b.revealed).into()
    }
}
