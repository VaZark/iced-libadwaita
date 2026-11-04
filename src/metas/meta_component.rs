//! `MetaComponent`: it owns no content; it chooses **which presentation** (variant) of one
//! piece of content to build, and builds only that one. Each variant has a key, a
//! [`Trigger`] and a closure that builds it. The first whose trigger fires wins; the last is the
//! fallback.
//!
//! Triggers:
//!
//! * **breakpoint**: [`Trigger::Breakpoint`] (made from a [`When`]) fires by the size of the
//!   *container* the component sits in, not the window;
//! * **select**: [`Trigger::Selected`] fires while the application has selected a key
//!   ([`selected`](MetaComponent::selected), controlled state), e.g. a search button;
//! * **groups**: `|` (any), `&` (all) and `!` (not) combine triggers, so a variant can answer to
//!   several things at once. There can be any number of alternatives, one trigger each.
//!
//! Other components are this one with their variants filled in:
//!
//! * [`ResponsiveTable`](crate::widgets::ResponsiveTable): a table while the columns fit, cards otherwise;
//! * [`SidebarList`](crate::widgets::SidebarList) with `cards_below`: compact rows, or cards.
//!
//! ```ignore
//! MetaComponent::new()
//!     .selected(self.searching.then_some(Bar::Search))      // on-select trigger
//!     .when_selected(Bar::Search, |_| search_entry())
//!     .variant(Bar::Compact, Trigger::selected(Bar::Menu) | !wide, |_| menu())  // a group
//!     .variant(Bar::Path, When::always(), |_| path_bar())   // fallback
//! ```
//!
//! The choice is the pure function [`MetaComponent::pick`] over `primitives::meta`. Like
//! [`responsive`](iced::widget::responsive), the result fills its container by default.

use crate::Element;
use crate::breakpoints::Breakpoints;
use crate::primitives::meta::{self, Trigger};
use iced::widget::responsive;
use iced::{Length, Size};
use std::rc::Rc;

type Build<'a, M> = Rc<dyn Fn(Size) -> Element<'a, M> + 'a>;

struct Variant<'a, P, M> {
    key: P,
    trigger: Trigger<P>,
    build: Build<'a, M>,
}

/// See the [module](self).
pub struct MetaComponent<'a, P, M> {
    variants: Vec<Variant<'a, P, M>>,
    selected: Vec<P>,
    width: Length,
    height: Length,
}

impl<'a, P: PartialEq + 'a, M: 'a> Default for MetaComponent<'a, P, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P: PartialEq + 'a, M: 'a> MetaComponent<'a, P, M> {
    pub fn new() -> Self {
        Self {
            variants: Vec::new(),
            selected: Vec::new(),
            width: Length::Fill,
            height: Length::Fill,
        }
    }
    /// Adds a presentation, in priority order. `trigger` is a [`Trigger`] or a
    /// [`When`](crate::primitives::When) (a breakpoint). `build` receives the container's size.
    /// List select-triggered variants before the breakpoint ones they override.
    pub fn variant(
        mut self,
        key: P,
        trigger: impl Into<Trigger<P>>,
        build: impl Fn(Size) -> Element<'a, M> + 'a,
    ) -> Self {
        self.variants.push(Variant {
            key,
            trigger: trigger.into(),
            build: Rc::new(build),
        });
        self
    }
    /// The on-select trigger: the key the application has selected, if any.
    pub fn selected(mut self, key: Option<P>) -> Self {
        self.selected = key.into_iter().collect();
        self
    }
    /// Several keys selected at once, for variants triggered by a group.
    pub fn selected_all(mut self, keys: impl IntoIterator<Item = P>) -> Self {
        self.selected = keys.into_iter().collect();
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    /// A variant shown while `key` itself is selected: `variant(key, Trigger::selected(key), ..)`.
    pub fn when_selected(self, key: P, build: impl Fn(Size) -> Element<'a, M> + 'a) -> Self
    where
        P: Clone,
    {
        let trigger = Trigger::selected(key.clone());
        self.variant(key, trigger, build)
    }

    /// The actions this component announces when it registers with a
    /// [`MetaParentComponent`](crate::metas::MetaParentComponent): the keys its variants are selected
    /// by, once each, in variant order.
    pub fn actions(&self) -> Vec<P>
    where
        P: Clone,
    {
        let mut keys: Vec<P> = Vec::new();
        for k in self.variants.iter().flat_map(|v| v.trigger.keys()) {
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        keys
    }

    fn index(&self, width: f32, scale: &Breakpoints) -> Option<usize> {
        meta::pick(
            self.variants.iter().map(|v| &v.trigger),
            &self.selected,
            width,
            scale,
        )
    }

    /// Builds the variant a container of `width` gets, **without measuring the container**, so the
    /// result takes its content's size and flows with the page. The normal conversion is built on
    /// [`responsive`](iced::widget::responsive), which fills its container and cannot shrink to
    /// its content's height. Use this where the content decides the height (a paragraph that
    /// swaps to an entry) and the width is known to the application, or irrelevant because the
    /// variants answer to [`selected`](Self::selected) rather than to a breakpoint: pass the
    /// window or pane width, or 0.
    pub fn flow(self, width: f32) -> Element<'a, M> {
        match self.index(width, &Breakpoints::current()) {
            Some(i) => (self.variants[i].build)(Size::new(width, 0.0)),
            None => iced::widget::space().into(),
        }
    }

    /// The key of the variant a container of `width` gets.
    pub fn pick(&self, width: f32, scale: &Breakpoints) -> Option<&P> {
        self.index(width, scale).map(|i| &self.variants[i].key)
    }
}

impl<'a, P: PartialEq + 'a, M: 'a> From<MetaComponent<'a, P, M>> for Element<'a, M> {
    fn from(m: MetaComponent<'a, P, M>) -> Self {
        let (w, h) = (m.width, m.height);
        responsive(
            move |size| match m.index(size.width, &Breakpoints::current()) {
                Some(i) => (m.variants[i].build)(size),
                None => iced::widget::space().into(),
            },
        )
        .width(w)
        .height(h)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breakpoints::WidthClass;
    use crate::primitives::When;

    const B: Breakpoints = Breakpoints {
        sm: 576.0,
        md: 768.0,
        lg: 992.0,
        xl: 1200.0,
        xxl: 1400.0,
    };

    fn meta() -> MetaComponent<'static, &'static str, ()> {
        let e = |_| iced::widget::space().into();
        MetaComponent::new()
            .variant("wide", When::min_width(900.0), e)
            .variant("mid", When::from_class(WidthClass::Md), e)
            .variant("narrow", When::always(), e)
    }

    #[test]
    fn the_first_variant_that_holds_wins() {
        let m = meta();
        assert_eq!(m.pick(1000.0, &B), Some(&"wide"));
        assert_eq!(m.pick(800.0, &B), Some(&"mid"));
        assert_eq!(m.pick(400.0, &B), Some(&"narrow"));
    }

    #[test]
    fn flow_builds_the_chosen_variant_without_a_container() {
        use std::cell::Cell;
        let built = Cell::new(None);
        let m: MetaComponent<'_, &str, ()> = MetaComponent::new()
            .variant("wide", When::min_width(900.0), |_| {
                built.set(Some("wide"));
                iced::widget::space().into()
            })
            .variant("narrow", When::always(), |_| {
                built.set(Some("narrow"));
                iced::widget::space().into()
            });
        let _ = m.flow(1000.0);
        assert_eq!(built.get(), Some("wide"));
    }

    #[test]
    fn the_last_variant_is_the_fallback() {
        let e = |_| iced::widget::space().into();
        let m: MetaComponent<'_, _, ()> = MetaComponent::new()
            .variant("a", When::min_width(500.0), e)
            .variant("b", When::min_width(300.0), e);
        assert_eq!(m.pick(100.0, &B), Some(&"b"));
        assert_eq!(MetaComponent::<(), ()>::new().pick(100.0, &B), None);
    }
}

#[cfg(test)]
mod select_tests {
    use super::*;
    use crate::primitives::When;

    #[test]
    fn selecting_a_variant_overrides_the_container() {
        let e = |_| iced::widget::space().into();
        let m = |sel| -> MetaComponent<'static, &'static str, ()> {
            MetaComponent::new()
                .selected(sel)
                .variant("search", Trigger::selected("search"), e)
                .variant("path", When::always(), e)
        };
        let b = Breakpoints::default();
        assert_eq!(m(None).pick(900.0, &b), Some(&"path"));
        assert_eq!(m(Some("search")).pick(900.0, &b), Some(&"search"));
    }
}
