//! `AdwToast` and `AdwToastOverlay`.

use super::Element;
use crate::style::{button as sb, container as sc};
use crate::Metrics;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::Length;

pub struct Toast<M> {
    title: String,
    action: Option<(String, M)>,
    on_dismiss: Option<M>,
}

impl<M> Toast<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: None,
            on_dismiss: None,
        }
    }
    /// `button-label` + `action-name`.
    pub fn action(mut self, label: impl Into<String>, on_press: M) -> Self {
        self.action = Some((label.into(), on_press));
        self
    }
    /// Message to send when the user dismisses the toast (adds a close button).
    pub fn on_dismiss(mut self, m: M) -> Self {
        self.on_dismiss = Some(m);
        self
    }
}

/// `AdwToastOverlay`: shows `toast` floating at the bottom of `content`.
/// The app is responsible for clearing it (e.g. after a timeout).
pub fn toast_overlay<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    toast: Option<Toast<M>>,
) -> Element<'a, M> {
    let Some(t) = toast else {
        return super::layered(content, None);
    };
    let m = Metrics::desktop();
    let mut r = row![text(t.title)].spacing(m.space_m).align_y(Vertical::Center);
    if let Some((label, msg)) = t.action {
        r = r.push(
            button(text(label))
                .padding(m.button_padding)
                .style(sb::osd)
                .on_press(msg),
        );
    }
    if let Some(msg) = t.on_dismiss {
        r = r.push(
            button(crate::Icon::WindowClose.colored(m.icon_size, iced::Color::WHITE))
                .padding(m.space_s)
                .style(sb::osd)
                .on_press(msg),
        );
    }
    let toast = container(r).padding([m.space_s, m.space_l]).style(sc::toast);
    // Slides up on show: 300ms, ADW_EASE (`SHOW_DURATION`). No hide animation: the app
    // removes the toast.
    let toast = super::entrance(toast, iced::Vector::new(0.0, 60.0))
        .trigger(crate::animation::Trigger::Toast)
        .spec(crate::animation::Spec::Timed(
            crate::animation::presets::TOAST_SHOW.0,
            crate::animation::presets::TOAST_SHOW.1,
        ));
    super::layered(
        content,
        Some(
            container(toast)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Bottom)
                .padding(m.space_xl)
                .into(),
        ),
    )
}
