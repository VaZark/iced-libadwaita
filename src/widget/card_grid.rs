//! `CardGrid`: cards that fit the width. As many columns as fit, each at most `max_width` wide,
//! and the cards fill their cells and keep their own height (rows are as tall as their tallest
//! card). Albums, playlists, apps, projects.
//!
//! A thin, named form of iced's `Grid::fluid`, with the sizing that suits cards.
//!
//! ```ignore
//! CardGrid::new(albums.iter().map(album_card)).max_width(180.0)
//! ```

use super::Element;
use iced::Length;
use iced::widget::grid::{self, Grid};

/// See the [module](self).
pub struct CardGrid<'a, M> {
    items: Vec<Element<'a, M>>,
    max_width: f32,
    spacing: f32,
    compact: bool,
}

impl<'a, M: 'a> CardGrid<'a, M> {
    pub fn new(items: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        Self {
            items: items.into_iter().collect(),
            max_width: 200.0,
            spacing: 12.0,
            compact: false,
        }
    }
    /// The widest a card may be; the number of columns follows from it (default 200).
    pub fn max_width(mut self, px: f32) -> Self {
        self.max_width = px.max(1.0);
        self
    }
    /// Lays the items out as a **list**: one column, tight, for compact [`MediaCard`](super::MediaCard)s
    /// (rows). The width of a column of rows is the whole width, so there is nothing to pick.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// The space between cards (default 12).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px.max(0.0);
        self
    }
}

impl<'a, M: 'a> From<CardGrid<'a, M>> for Element<'a, M> {
    fn from(g: CardGrid<'a, M>) -> Self {
        if g.compact {
            return iced::widget::column(g.items)
                .spacing(g.spacing.min(2.0))
                .width(Length::Fill)
                .into();
        }
        Grid::with_children(g.items)
            .fluid(g.max_width)
            .spacing(g.spacing)
            .height(grid::Sizing::EvenlyDistribute(Length::Shrink))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_sizes() {
        let g: CardGrid<'_, ()> = CardGrid::new([])
            .max_width(160.0)
            .spacing(8.0)
            .compact(true);
        assert_eq!((g.max_width, g.spacing, g.compact), (160.0, 8.0, true));
    }
}
