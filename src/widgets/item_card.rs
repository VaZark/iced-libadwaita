//! `ItemCard`: an album, an artist, a playlist, an app: a picture, a title and a subtitle, drawn
//! as a **card** or, in compact mode, as a tight [`ItemRow`](crate::widgets::ItemRow). One pattern in
//! two densities, so a grid of cards and a list of rows are the same data with one switch.
//!
//! * **Comfort** (default): the picture centered over the title, the subtitle on the card's bottom
//!   edge, the whole card pressable.
//! * **Compact**: the picture small at the start and the title over the subtitle beside it, a
//!   flat row ([`flat_row`](crate::styles::button::flat_row)).
//!
//! The density is the item's own. How items are arranged is the container's: a
//! [`FlowGrid`](crate::layouts::FlowGrid) with several columns makes a grid, with one column a list, and a
//! [`TableGrid`](crate::layouts::TableGrid) gives each its span.
//!
//! ```ignore
//! FlowGrid::new(albums.iter().map(|a| {
//!     ItemCard::new(&a.name)
//!         .subtitle(format!("{} · {}", a.artist, a.year))
//!         .picture(cover(a.id, if density == Density::Compact { 36.0 } else { 88.0 }))
//!         .density(density)
//!         .on_press(Msg::Open(a.id))
//!         .into()
//! }))
//! .cols(if density == Density::Compact { 1 } else { 4 })
//! ```

use super::item_row::ItemRowProps;
use crate::Element;
use crate::Metrics;
use crate::components::typography as ty;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::config::Density;
use crate::styles::text as st;
use crate::widgets::ItemRow;
use iced::Length;
use iced::widget::text::Wrapping;
use iced::widget::{column, container};

/// See the [module](self).
pub struct ItemCard<'a, M> {
    row_props: ItemRowProps,
    title_props: TextProps,
    subtitle_props: TextProps,
    picture_props: ContainerProps,
    action_props: ButtonProps,
    container_props: ContainerProps,
    title: String,
    subtitle: Option<String>,
    picture: Option<Element<'a, M>>,
    density: Density,
    on_press: Option<M>,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    picture_align: iced::alignment::Horizontal,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> ItemCard<'a, M> {
    pub fn row_props(mut self, props: ItemRowProps) -> Self {
        self.row_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.subtitle_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn picture_props(mut self, props: ContainerProps) -> Self {
        self.picture_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }

    pub fn new(title: impl Into<String>) -> Self {
        Self {
            row_props: ItemRowProps::default(),
            title_props: TextProps::default(),
            subtitle_props: TextProps::default(),
            picture_props: ContainerProps::default(),
            action_props: ButtonProps::default(),
            container_props: ContainerProps::default(),

            title: title.into(),
            subtitle: None,
            picture: None,
            density: Density::Comfort,
            on_press: None,
            spacing: None,
            padding: None,
            picture_align: iced::alignment::Horizontal::Center,
            metrics: Metrics::desktop(),
        }
    }
    /// The dimmed line under the title: the artist, the year, a count.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    /// The gap between the picture, the title and the subtitle of a card (default: the theme's
    /// `space_xs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The padding inside a card (default: the theme's `card_padding`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// Where the picture sits across the card (default `Horizontal::Center`).
    pub fn picture_align(mut self, a: iced::alignment::Horizontal) -> Self {
        self.picture_align = a;
        self
    }
    /// The picture: a cover, an avatar. Make it big for a card and small for the compact row.
    pub fn picture(mut self, picture: impl Into<Element<'a, M>>) -> Self {
        self.picture = Some(picture.into());
        self
    }
    /// [`Comfort`](Density::Comfort): a card. [`Compact`](Density::Compact): a tight row.
    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }
    pub fn on_press(mut self, m: M) -> Self {
        self.on_press = Some(m);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<ItemCard<'a, M>> for Element<'a, M> {
    fn from(c: ItemCard<'a, M>) -> Self {
        let m = c.metrics;
        if c.density == Density::Compact {
            let mut row = ItemRow::new(c.title)
                .metrics(m)
                .inset()
                .title_props(c.title_props.clone())
                .subtitle_props(c.subtitle_props.clone())
                .container_props(c.container_props.clone());
            if let Some(s) = c.subtitle {
                row = row.subtitle(s);
            }
            if let Some(p) = c.picture {
                row = row.picture(p);
            }
            return c
                .action_props
                .apply(
                    iced::widget::button(c.row_props.apply(row))
                        .padding(0)
                        .width(Length::Fill)
                        .style(crate::styles::button::flat_row(8.0, 0.08, 0.14))
                        .on_press_maybe(c.on_press),
                )
                .into();
        }
        let mut body = column![]
            .spacing(c.spacing.unwrap_or(m.space_xs))
            .width(Length::Fill)
            .height(Length::Fill);
        if let Some(p) = c.picture {
            body = body.push(
                c.picture_props
                    .apply(container(p).width(Length::Fill).align_x(c.picture_align)),
            );
        }
        // Text stays inside the card: it wraps at words (and, for one word wider than the card,
        // at characters). `heading` alone never wraps.
        body = body.push(
            c.title_props.apply(
                ty::heading(c.title)
                    .wrapping(Wrapping::WordOrGlyph)
                    .width(Length::Fill),
            ),
        );
        if let Some(s) = c.subtitle {
            // The subtitle (the artist and year, the count) sits on the card's bottom edge: in a
            // grid with a shared row height the cards' last lines line up.
            body = body.push(iced::widget::space::vertical());
            body = body.push(
                c.subtitle_props.apply(
                    ty::caption(s)
                        .style(st::dim)
                        .wrapping(Wrapping::WordOrGlyph)
                        .width(Length::Fill),
                ),
            );
        }
        // The card draws its own surface; the button only adds the events.
        let surface = c.container_props.apply(
            container(body)
                .padding(c.padding.unwrap_or(m.card_padding))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(crate::styles::container::card),
        );
        c.action_props
            .apply(
                iced::widget::button(surface)
                    .padding(0)
                    .style(crate::styles::button::events)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .on_press_maybe(c.on_press),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_takes_spacing_padding_and_picture_alignment() {
        let c = ItemCard::<u8>::new("A")
            .subtitle("b")
            .picture(iced::widget::space())
            .spacing(3.0)
            .padding(5.0)
            .picture_align(iced::alignment::Horizontal::Left);
        assert_eq!(
            (c.spacing, c.padding),
            (Some(3.0), Some(iced::Padding::from(5.0)))
        );
        let _: Element<'_, u8> = c.into();
    }

    #[test]
    fn the_builder_keeps_its_parts() {
        let c: ItemCard<'_, u8> = ItemCard::new("Album")
            .subtitle("Artist")
            .picture(iced::widget::space())
            .density(Density::Compact)
            .on_press(3);
        assert!(c.density == Density::Compact && c.picture.is_some() && c.on_press == Some(3));
        assert_eq!(c.subtitle.as_deref(), Some("Artist"));
        assert_eq!(c.title, "Album");
    }
}

/// Reusable, no-op-by-default presentation for [`ItemCard`].
#[derive(Clone, Default)]
pub struct ItemCardProps {
    pub row_props: Option<ItemRowProps>,
    pub title_props: Option<TextProps>,
    pub subtitle_props: Option<TextProps>,
    pub picture_props: Option<ContainerProps>,
    pub action_props: Option<ButtonProps>,
    pub container_props: Option<ContainerProps>,
}
impl ItemCardProps {
    pub fn row_props(mut self, props: ItemRowProps) -> Self {
        self.row_props = Some(props);
        self
    }

    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.subtitle_props = Some(props);
        self
    }
    pub fn picture_props(mut self, props: ContainerProps) -> Self {
        self.picture_props = Some(props);
        self
    }
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = Some(props);
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }
    pub fn apply<'a, M: Clone + 'a>(&self, widget: ItemCard<'a, M>) -> ItemCard<'a, M> {
        let widget = if let Some(props) = &self.row_props {
            widget.row_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.title_props {
            widget.title_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.subtitle_props {
            widget.subtitle_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.picture_props {
            widget.picture_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.action_props {
            widget.action_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.container_props {
            widget.container_props(props.clone())
        } else {
            widget
        };
        widget
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    #[test]
    fn action_geometry_is_applied_in_both_densities_without_replacing_press() {
        for density in [Density::Comfort, Density::Compact] {
            let props = ItemCardProps::default()
                .title_props(TextProps::default().size(23))
                .action_props(ButtonProps::default().width(123).height(45));
            let card = props.apply(
                ItemCard::new("Album")
                    .subtitle("Artist")
                    .density(density)
                    .on_press(7u8),
            );
            assert_eq!(card.on_press, Some(7));
            assert_eq!(card.title, "Album");
            assert_eq!(card.subtitle.as_deref(), Some("Artist"));
            let element: Element<'_, u8> = card.into();
            assert_eq!(
                element.as_widget().size(),
                iced::Size::new(Length::Fixed(123.0), Length::Fixed(45.0))
            );
        }
    }

    use super::*;
    use iced::advanced::widget::Tree;
    fn same_tree(a: &Tree, b: &Tree) {
        assert!(a.tag == b.tag);
        assert_eq!(a.children.len(), b.children.len());
        for (a, b) in a.children.iter().zip(&b.children) {
            same_tree(a, b);
        }
    }
    #[test]
    fn reusable_defaults_preserve_geometry_and_tree() {
        let build = || ItemCard::new("Title").subtitle("Body").on_press(7u8);
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = ItemCardProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
