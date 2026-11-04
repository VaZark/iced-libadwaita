//! Headless logic of a meta component: which variant is shown, and what triggers it.
//!
//! A variant is shown when its [`Trigger`] fires:
//!
//! * [`Trigger::Breakpoint`]: the container is wide enough ([`When`]), the responsive trigger;
//! * [`Trigger::Select`]: the application selected this variant (controlled), the on-select
//!   trigger, e.g. a search button that turns a path bar into a search bar.
//!
//! The first variant whose trigger fires wins, so list select-triggered variants before the
//! breakpoint ones they override. When none fires, the last variant is the fallback.

use super::responsive::When;
use crate::breakpoints::Breakpoints;
use std::ops::{BitAnd, BitOr, Not};

/// When a variant is shown. Triggers combine, so one variant can answer to several things and
/// the combinations stay one value:
///
/// ```ignore
/// // searching, or any narrow container
/// Trigger::selected(Bar::Search) | When::below_class(WidthClass::Md).into()
/// // wide AND not filtering
/// Trigger::from(When::from_class(WidthClass::Lg)) & !Trigger::selected(Mode::Filter)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Trigger<K> {
    /// The container satisfies the condition (the responsive trigger).
    Breakpoint(When),
    /// The application has selected this key (the on-select trigger, controlled state).
    Selected(K),
    /// Any of the group fires (OR).
    Any(Vec<Trigger<K>>),
    /// Every one of the group fires (AND).
    All(Vec<Trigger<K>>),
    /// The inner trigger does not fire.
    Not(Box<Trigger<K>>),
}

impl<K> From<When> for Trigger<K> {
    fn from(w: When) -> Self {
        Self::Breakpoint(w)
    }
}

impl<K> Trigger<K> {
    /// Always fires (a fallback).
    pub const fn always() -> Self {
        Self::Breakpoint(When::always())
    }
    pub fn selected(key: K) -> Self {
        Self::Selected(key)
    }
    pub fn any(group: impl IntoIterator<Item = Trigger<K>>) -> Self {
        Self::Any(group.into_iter().collect())
    }
    pub fn all(group: impl IntoIterator<Item = Trigger<K>>) -> Self {
        Self::All(group.into_iter().collect())
    }
}

impl<K: Clone> Trigger<K> {
    /// The keys whose selection can make this trigger fire: every `Selected` key outside a
    /// `Not`. This is what a variant announces to a
    /// [`MetaParentComponent`](crate::metas::MetaParentComponent) when it registers.
    pub fn keys(&self) -> Vec<K> {
        match self {
            Self::Breakpoint(_) | Self::Not(_) => Vec::new(),
            Self::Selected(k) => vec![k.clone()],
            Self::Any(g) | Self::All(g) => g.iter().flat_map(Self::keys).collect(),
        }
    }
}

impl<K: PartialEq> Trigger<K> {
    /// Whether it fires for a container of `width` when the application has `selected` these
    /// keys. An empty `Any` never fires; an empty `All` always does.
    pub fn fires(&self, selected: &[K], width: f32, scale: &Breakpoints) -> bool {
        match self {
            Self::Breakpoint(w) => w.holds(width, scale),
            Self::Selected(k) => selected.contains(k),
            Self::Any(g) => g.iter().any(|t| t.fires(selected, width, scale)),
            Self::All(g) => g.iter().all(|t| t.fires(selected, width, scale)),
            Self::Not(t) => !t.fires(selected, width, scale),
        }
    }
}

impl<K> BitOr for Trigger<K> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Any(mut a), Self::Any(b)) => {
                a.extend(b);
                Self::Any(a)
            }
            (Self::Any(mut a), b) => {
                a.push(b);
                Self::Any(a)
            }
            (a, b) => Self::Any(vec![a, b]),
        }
    }
}

impl<K> BitAnd for Trigger<K> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::All(mut a), Self::All(b)) => {
                a.extend(b);
                Self::All(a)
            }
            (Self::All(mut a), b) => {
                a.push(b);
                Self::All(a)
            }
            (a, b) => Self::All(vec![a, b]),
        }
    }
}

impl<K> Not for Trigger<K> {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Not(t) => *t,
            t => Self::Not(Box::new(t)),
        }
    }
}

/// The index of the variant to show: the first whose trigger fires, else the last, else `None`
/// when there are no variants.
pub fn pick<'a, K: PartialEq + 'a>(
    mut triggers: impl ExactSizeIterator<Item = &'a Trigger<K>>,
    selected: &[K],
    width: f32,
    scale: &Breakpoints,
) -> Option<usize> {
    let n = triggers.len();
    triggers
        .position(|t| t.fires(selected, width, scale))
        .or_else(|| n.checked_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breakpoints::WidthClass;

    const B: Breakpoints = Breakpoints {
        sm: 576.0,
        md: 768.0,
        lg: 992.0,
        xl: 1200.0,
        xxl: 1400.0,
    };

    type T = Trigger<&'static str>;

    fn run(v: &[(&'static str, T)], sel: &[&'static str], w: f32) -> Option<&'static str> {
        pick(v.iter().map(|(_, t)| t), sel, w, &B).map(|i| v[i].0)
    }

    fn wide() -> T {
        When::from_class(WidthClass::Md).into()
    }

    #[test]
    fn breakpoints_choose_when_nothing_is_selected() {
        let v = [("wide", wide()), ("narrow", T::always())];
        assert_eq!(run(&v, &[], 900.0), Some("wide"));
        assert_eq!(run(&v, &[], 400.0), Some("narrow"));
    }

    #[test]
    fn a_selected_variant_overrides_the_breakpoints() {
        let v = [("search", T::selected("search")), ("path", T::always())];
        assert_eq!(run(&v, &[], 900.0), Some("path"));
        assert_eq!(run(&v, &["search"], 100.0), Some("search"));
    }

    #[test]
    fn several_alternatives_each_have_their_own_trigger() {
        let v = [
            ("search", T::selected("search")),
            ("filter", T::selected("filter")),
            ("path", T::always()),
        ];
        assert_eq!(run(&v, &["filter"], 900.0), Some("filter"));
        assert_eq!(run(&v, &["search"], 900.0), Some("search"));
    }

    #[test]
    fn grouped_triggers_answer_to_any_or_all() {
        // One variant for: searching, OR a narrow container.
        let any = T::selected("search") | !wide();
        assert!(any.fires(&["search"], 900.0, &B));
        assert!(any.fires(&[], 300.0, &B));
        assert!(!any.fires(&[], 900.0, &B));
        // Wide AND not filtering.
        let all = wide() & !T::selected("filter");
        assert!(all.fires(&[], 900.0, &B));
        assert!(!all.fires(&["filter"], 900.0, &B));
        assert!(!all.fires(&[], 300.0, &B));
    }

    #[test]
    fn chained_groups_stay_flat_and_empty_groups_behave() {
        let t = T::selected("a") | T::selected("b") | T::selected("c");
        assert!(matches!(&t, Trigger::Any(g) if g.len() == 3));
        assert!(!T::any([]).fires(&[], 0.0, &B));
        assert!(T::all([]).fires(&[], 0.0, &B));
        assert_eq!(!!T::selected("a"), T::selected("a"));
    }

    #[test]
    fn keys_are_the_selections_that_can_fire_it() {
        let t = T::selected("a") | (wide() & T::selected("b")) | !T::selected("c");
        assert_eq!(t.keys(), vec!["a", "b"]);
        assert!(T::always().keys().is_empty());
    }

    #[test]
    fn the_last_variant_is_the_fallback_and_empty_is_none() {
        let v = [("a", T::selected("a")), ("b", T::selected("b"))];
        assert_eq!(run(&v, &[], 100.0), Some("b"));
        assert_eq!(run(&[], &[], 100.0), None);
    }
}
