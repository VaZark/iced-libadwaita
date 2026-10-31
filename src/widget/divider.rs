//! A hairline between things: the one divider the other widgets' separators can share.

use super::Element;
use crate::style::container as sc;
use iced::Length;
use iced::widget::{container, space};

/// Which theme color the hairline uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DividerTone {
    /// `--border-color`.
    #[default]
    Border,
    /// Between boxed-list rows (`--card-shade-color`).
    Card,
    /// Under or over bars (`--headerbar-shade-color`).
    Bar,
    /// Between popover sections (`--popover-shade-color`).
    Popover,
    /// Beside a sidebar (`--sidebar-border-color`).
    Sidebar,
}

/// A 1px line, horizontal by default.
pub struct Divider {
    vertical: bool,
    tone: DividerTone,
    spacing: f32,
}

pub fn divider() -> Divider {
    Divider {
        vertical: false,
        tone: DividerTone::Border,
        spacing: 0.0,
    }
}

impl Divider {
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
    pub fn tone(mut self, tone: DividerTone) -> Self {
        self.tone = tone;
        self
    }
    /// Empty space on both sides of the line, across its direction.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<'a, M: 'a> From<Divider> for Element<'a, M> {
    fn from(d: Divider) -> Self {
        let style = match d.tone {
            DividerTone::Border => sc::separator,
            DividerTone::Card => sc::card_separator,
            DividerTone::Bar => sc::bar_separator,
            DividerTone::Popover => sc::popover_separator,
            DividerTone::Sidebar => sc::sidebar_separator,
        };
        if d.vertical {
            container(
                container(space().width(1))
                    .height(Length::Fill)
                    .style(style),
            )
            .padding([0.0, d.spacing])
            .into()
        } else {
            container(
                container(space().height(1))
                    .width(Length::Fill)
                    .style(style),
            )
            .padding([d.spacing, 0.0])
            .into()
        }
    }
}
