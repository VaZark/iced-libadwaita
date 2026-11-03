//! `Tabs`: a row of tabs and the panel of the active one, after Mantine's
//! [Tabs](https://mantine.dev/core/tabs/) and Radix UI's. The behavior (which tab a click
//! activates, disabled tabs) is the [`tabs`](crate::primitives::tabs) primitive, shared with the
//! segmented control behind [`toggle_group`](super::toggle_group).
//!
//! Controlled: the app owns the active key and takes it from `on_change`. Each tab may carry
//! the panel that shows while it is active (built only then), or none, for a list that drives
//! something else (the gallery's Preview / Code tabs).
//!
//! ```ignore
//! Tabs::new(Some(self.tab), Msg::Tab)
//!     .variant(TabsVariant::Floating)
//!     .tab(TabItem::new(Tab::Gallery, "Gallery").icon(Icon::Home).panel(|| gallery()))
//!     .tab(TabItem::new(Tab::Messages, "Messages").panel(|| messages()))
//!     .tab(TabItem::new(Tab::Settings, "Settings").disabled(true))
//! ```
//!
//! Variants, as in Mantine: [`Default`](TabsVariant::Default) (an accent line under the active
//! tab, over a hairline), [`Outline`](TabsVariant::Outline) (the active tab boxed),
//! [`Pills`](TabsVariant::Pills) (the active tab filled) and
//! [`Floating`](TabsVariant::Floating) (a highlight that slides behind the active tab, a
//! [`FloatingIndicator`](super::FloatingIndicator)). Arrow-key navigation is not included: iced
//! has no focus model to hang it on (the primitive has `step` for it).

use super::{Element, FloatingIndicator};
use crate::primitives::tabs::TabList;
use crate::style::button as sb;
use crate::style::container as sc;
use crate::style::text as st;
use crate::widget::typography::label;
use crate::{Icon, Metrics, Theme};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space};

/// How the tabs look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsVariant {
    /// An accent line under the active tab, over a hairline.
    #[default]
    Default,
    /// The active tab in a box that opens into its panel.
    Outline,
    /// The active tab filled with the accent color.
    Pills,
    /// A highlight that slides behind the active tab.
    Floating,
}

type Panel<'a, M> = Box<dyn Fn() -> Element<'a, M> + 'a>;

/// One tab: its key, label, optional icon and panel.
pub struct TabItem<'a, K, M> {
    key: K,
    label: String,
    icon: Option<Icon>,
    disabled: bool,
    panel: Option<Panel<'a, M>>,
}

impl<'a, K, M> TabItem<'a, K, M> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            icon: None,
            disabled: false,
            panel: None,
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A disabled tab is dimmed and cannot be activated.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// What shows while this tab is active; built only then.
    pub fn panel(mut self, build: impl Fn() -> Element<'a, M> + 'a) -> Self {
        self.panel = Some(Box::new(build));
        self
    }
}

/// See the [module](self).
pub struct Tabs<'a, K, M> {
    items: Vec<TabItem<'a, K, M>>,
    active: Option<K>,
    on_change: Box<dyn Fn(K) -> M + 'a>,
    variant: TabsVariant,
    grow: bool,
    metrics: Metrics,
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> Tabs<'a, K, M> {
    pub fn new(active: Option<K>, on_change: impl Fn(K) -> M + 'a) -> Self {
        Self {
            items: Vec::new(),
            active,
            on_change: Box::new(on_change),
            variant: TabsVariant::default(),
            grow: false,
            metrics: Metrics::desktop(),
        }
    }
    pub fn tab(mut self, item: TabItem<'a, K, M>) -> Self {
        self.items.push(item);
        self
    }
    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self
    }
    /// The tabs share the width equally (not for `Floating`, which hugs its tabs).
    pub fn grow(mut self, grow: bool) -> Self {
        self.grow = grow;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn list(&self) -> TabList {
        TabList::with_disabled(
            self.items.len(),
            self.items.iter().map(|t| t.disabled).collect(),
        )
    }
    fn active_index(&self) -> Option<usize> {
        let key = self.active.as_ref()?;
        self.items.iter().position(|t| t.key == *key)
    }
    /// The message a press on tab `i` sends: none when it is disabled.
    fn press(&self, i: usize) -> Option<M> {
        self.list()
            .activate(i)
            .map(|i| (self.on_change)(self.items[i].key.clone()))
    }

    /// One tab's button, in the look of the variant.
    fn tab_button(&self, i: usize, active: bool) -> Element<'a, M> {
        let m = &self.metrics;
        let item = &self.items[i];
        let text = |s: String| {
            let t = label(s);
            if active && !item.disabled {
                t
            } else {
                t.style(st::dim)
            }
        };
        let mut content = row![].spacing(m.space_s).align_y(Vertical::Center);
        if let Some(icon) = item.icon {
            content = content.push(icon.view(m.icon_size));
        }
        content = content.push(text(item.label.clone()));
        let b = button(content)
            .padding([m.space_xs, m.space_m])
            .on_press_maybe(self.press(i));
        match self.variant {
            TabsVariant::Pills if active => b.style(sb::suggested_pill),
            _ => b.style(sb::flat),
        }
        .into()
    }

    fn tab_list(&self) -> Element<'a, M> {
        let m = &self.metrics;
        let active = self.active_index();
        if self.variant == TabsVariant::Floating {
            let tabs = (0..self.items.len())
                .map(|i| self.tab_button(i, active == Some(i)))
                .collect();
            return FloatingIndicator::new(tabs, active.unwrap_or(0))
                .spacing(m.space_xs)
                .into();
        }
        let width = if self.grow {
            Length::Fill
        } else {
            Length::Shrink
        };
        let mut tabs = row![].spacing(m.space_xs);
        for i in 0..self.items.len() {
            let on = active == Some(i);
            let button = self.tab_button(i, on);
            let tab: Element<'a, M> = match self.variant {
                TabsVariant::Default => {
                    let line =
                        container(space().height(2))
                            .width(Length::Fill)
                            .style(move |t: &Theme| iced::widget::container::Style {
                                background: on
                                    .then_some(iced::Background::Color(t.semantic.accent.bg)),
                                ..Default::default()
                            });
                    column![button, line].into()
                }
                TabsVariant::Outline => container(button)
                    .style(move |t: &Theme| iced::widget::container::Style {
                        border: if on {
                            iced::Border {
                                color: t.semantic.border,
                                width: 1.0,
                                radius: iced::border::top(t.metrics.radius_m),
                            }
                        } else {
                            iced::Border::default()
                        },
                        ..Default::default()
                    })
                    .into(),
                _ => button,
            };
            tabs = tabs.push(container(tab).width(width));
        }
        let tabs = tabs.width(if self.grow {
            Length::Fill
        } else {
            Length::Shrink
        });
        match self.variant {
            TabsVariant::Default | TabsVariant::Outline => column![
                tabs,
                container(space().height(1))
                    .width(Length::Fill)
                    .style(sc::card_separator)
            ]
            .into(),
            _ => tabs.into(),
        }
    }
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> From<Tabs<'a, K, M>> for Element<'a, M> {
    fn from(t: Tabs<'a, K, M>) -> Self {
        let list = t.tab_list();
        let panel = t
            .active_index()
            .and_then(|i| t.items[i].panel.as_ref())
            .map(|build| build());
        match panel {
            Some(p) => column![list, p].spacing(t.metrics.space_m).into(),
            None => list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs(active: Option<u8>) -> Tabs<'static, u8, u8> {
        Tabs::new(active, |k| k)
            .tab(TabItem::new(0, "One"))
            .tab(TabItem::new(1, "Two").disabled(true))
            .tab(TabItem::new(2, "Three"))
    }

    #[test]
    fn a_tab_sends_its_key_and_a_disabled_one_nothing() {
        let t = tabs(Some(0));
        assert_eq!(t.press(0), Some(0));
        assert_eq!(t.press(1), None);
        assert_eq!(t.press(2), Some(2));
        assert_eq!(t.press(7), None, "no such tab");
    }

    #[test]
    fn the_active_tab_is_found_by_key() {
        assert_eq!(tabs(Some(2)).active_index(), Some(2));
        assert_eq!(tabs(None).active_index(), None);
        assert_eq!(tabs(Some(9)).active_index(), None);
    }

    #[test]
    fn only_the_active_tabs_panel_is_built() {
        use std::cell::Cell;
        let built = Cell::new(0);
        let t: Tabs<'_, u8, u8> = Tabs::new(Some(1), |k| k)
            .tab(TabItem::new(0, "A").panel(|| {
                built.set(built.get() + 10);
                iced::widget::space().into()
            }))
            .tab(TabItem::new(1, "B").panel(|| {
                built.set(built.get() + 1);
                iced::widget::space().into()
            }));
        let _: Element<'_, u8> = t.into();
        assert_eq!(built.get(), 1);
    }
}
