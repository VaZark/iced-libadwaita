//! `AdwBanner`.

use crate::Element;
use crate::Metrics;
use crate::styles::{button as sb, container as sc};
use iced::Length;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};

pub struct Banner<M> {
    title: String,
    button_label: Option<String>,
    on_button: Option<M>,
    revealed: bool,
    text_color: Option<iced::Color>,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    button_padding: Option<iced::Padding>,
    title_align: Horizontal,
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
            spacing: None,
            padding: None,
            button_padding: None,
            title_align: Horizontal::Center,
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
    /// The gap between the title and the button (default: the theme's `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the banner's content (default: the theme's `space_s` above and below,
    /// `space_m` at the sides).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The padding of the button (default: the theme's `button_padding`).
    pub fn button_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.button_padding = Some(p.into());
        self
    }
    /// Where the title sits in the space the button leaves (default `Horizontal::Center`).
    pub fn title_align(mut self, a: Horizontal) -> Self {
        self.title_align = a;
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
            container(text(b.title))
                .width(Length::Fill)
                .align_x(b.title_align)
        ]
        .align_y(Vertical::Center)
        .spacing(b.spacing.unwrap_or(m.space_m));
        if let (Some(l), Some(msg)) = (b.button_label, b.on_button) {
            r = r.push(
                button(text(l))
                    .padding(b.button_padding.unwrap_or(m.button_padding))
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
            .padding(b.padding.unwrap_or([m.space_s, m.space_m].into()))
            .width(Length::Fill)
            .style(move |t: &crate::Theme| {
                let mut st = sc::banner(t);
                if let Some(c) = override_fg {
                    st.text_color = Some(c);
                }
                st
            });
        // Hidden by default; `revealed` slides it in/out (250ms, ADW_EASE).
        crate::layouts::reveal(banner, b.revealed).into()
    }
}

#[cfg(test)]
mod spacing_params_tests {
    use super::*;

    #[test]
    fn a_banner_takes_spacing_padding_and_title_alignment() {
        let b = Banner::<u8>::new("Hi")
            .button("Go", 1)
            .spacing(3.0)
            .padding([1.0, 2.0])
            .button_padding(4.0)
            .title_align(Horizontal::Left);
        assert_eq!(b.spacing, Some(3.0));
        assert_eq!(b.padding, Some(iced::Padding::from([1.0, 2.0])));
        assert_eq!(b.title_align, Horizontal::Left);
        let _: Element<'_, u8> = b.into();
    }
}
