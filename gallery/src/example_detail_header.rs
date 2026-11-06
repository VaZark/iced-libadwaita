//! `DetailHeader`: the head of a detail page (a playlist, an album, an artist), after the usual
//! music-app pattern.
//!
//! * **Wide**: the cover on the left, a small kind label, the large title, the description and
//!   facts beside it, centered against the cover, and an action row under both: the main action first, then the options,
//!   then something at the far end.
//! * **Narrow** ([`narrow`](DetailHeader::narrow)): the cover and the title centered, then the
//!   description and facts, and one action row with the options at the start and the main
//!   action at the end.
//! * A [wash](crate::styles::container::wash) of [`tint`](DetailHeader::tint) fades down from the
//!   top.
//!
//! The application decides which arrangement from its width (for example
//! `WidthClass::of(width) < WidthClass::Md`).
//!
//! ```ignore
//! DetailHeader::new("Evening drive")
//!     .kind("Playlist")
//!     .cover(cover)
//!     .description(ty::body("Slow songs for a long road."))
//!     .facts(ty::caption("24 songs, 1 hr 32 min"))
//!     .primary(suggested_circular_button(Icon::Play.view(16.0)).on_press(Msg::Play))
//!     .action(menu_button(Icon::ViewMore, menu))
//!     .tint(cover_color)
//!     .narrow(width < 576.0)
//! ```

use iced::widget::{column, container, row, space, text};
use iced::{Alignment, Color, Length};
use iced_adaptive_design::Element;
use iced_adaptive_design::components::typography as ty;
use iced_adaptive_design::styles::container as sc;
use iced_adaptive_design::{Metrics, Theme};

/// See the [module](self).
pub struct DetailHeader<'a, M> {
    title: String,
    kind: Option<String>,
    cover: Option<Element<'a, M>>,
    description: Option<Element<'a, M>>,
    facts: Option<Element<'a, M>>,
    primary: Option<Element<'a, M>>,
    actions: Vec<Element<'a, M>>,
    end: Option<Element<'a, M>>,
    tint: Option<Color>,
    narrow: bool,
    metrics: Metrics,
}

impl<'a, M: 'a> DetailHeader<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: None,
            cover: None,
            description: None,
            facts: None,
            primary: None,
            actions: Vec::new(),
            end: None,
            tint: None,
            narrow: false,
            metrics: Metrics::desktop(),
        }
    }
    /// The small label over the title: "Playlist", "Album", "Artist".
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }
    /// The artwork.
    pub fn cover(mut self, cover: impl Into<Element<'a, M>>) -> Self {
        self.cover = Some(cover.into());
        self
    }
    pub fn description(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.description = Some(e.into());
        self
    }
    /// A line of facts under the description: counts, durations, dates.
    pub fn facts(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.facts = Some(e.into());
        self
    }
    /// The main action (play). First in the wide action row, last in the narrow one.
    pub fn primary(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.primary = Some(e.into());
        self
    }
    /// An option next to the main action (a menu, a favorite toggle). Add as many as needed.
    pub fn action(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.actions.push(e.into());
        self
    }
    /// Something pushed to the far end of the wide action row.
    pub fn end(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.end = Some(e.into());
        self
    }
    /// The color of the wash behind the header (usually the cover's dominant color). Without
    /// one there is no wash.
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }
    /// The narrow arrangement (default wide).
    pub fn narrow(mut self, narrow: bool) -> Self {
        self.narrow = narrow;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<DetailHeader<'a, M>> for Element<'a, M> {
    fn from(h: DetailHeader<'a, M>) -> Self {
        let m = h.metrics;
        let tint = h.tint;
        let wash = move |t: &Theme| match tint {
            Some(c) => sc::wash(c)(t),
            None => container::Style::default(),
        };
        let blank = || -> Element<'a, M> { space().into() };
        let mut actions = row![].spacing(m.space_m).align_y(Alignment::Center);
        if h.narrow {
            for a in h.actions {
                actions = actions.push(a);
            }
            actions = actions.push(space::horizontal());
            if let Some(p) = h.primary {
                actions = actions.push(p);
            }
            let mut col = column![].spacing(m.space_m);
            if let Some(c) = h.cover {
                col = col.push(container(c).center_x(Length::Fill));
            }
            col = col.push(
                ty::title_2(h.title)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
            );
            col = col.push(h.description.unwrap_or_else(blank));
            if let Some(f) = h.facts {
                col = col.push(f);
            }
            return container(col.push(actions))
                .padding(m.page_padding.left + 4.0)
                .width(Length::Fill)
                .style(wash)
                .into();
        }
        if let Some(p) = h.primary {
            actions = actions.push(p);
        }
        for a in h.actions {
            actions = actions.push(a);
        }
        actions = actions.push(space::horizontal());
        if let Some(e) = h.end {
            actions = actions.push(e);
        }
        let mut text_col = column![].spacing(m.space_s);
        if let Some(k) = h.kind {
            text_col = text_col.push(ty::caption(k));
        }
        text_col = text_col.push(text(h.title).size(48).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..iced::Font::DEFAULT
        }));
        if let Some(d) = h.description {
            text_col = text_col.push(d);
        }
        if let Some(f) = h.facts {
            text_col = text_col.push(f);
        }
        // The text is centered against the cover, whichever is taller.
        let mut top = row![]
            .spacing(m.space_xl)
            .height(Length::Shrink)
            .align_y(Alignment::Center);
        if let Some(c) = h.cover {
            top = top.push(c);
        }
        top = top.push(text_col.width(Length::Fill));
        container(column![top, actions].spacing(m.space_l))
            .padding(m.page_padding.left * 2.0)
            .width(Length::Fill)
            .style(wash)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_parts_in_order() {
        let h: DetailHeader<'_, ()> = DetailHeader::new("Mix")
            .kind("Playlist")
            .action(space())
            .action(space())
            .narrow(true);
        assert_eq!(h.actions.len(), 2);
        assert!(h.narrow && h.kind.as_deref() == Some("Playlist"));
        assert!(h.tint.is_none());
    }
}
