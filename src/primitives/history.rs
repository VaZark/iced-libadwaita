//! Navigation history: where Back goes. **Open** pushes the place you are leaving, so Back
//! returns to it; **Go** jumps to a top-level place (a sidebar entry) and forgets the trail;
//! **Back** pops. Back returns to the place you came from, never to a fixed parent, so opening an
//! album from a search and pressing Back lands on the search.
//!
//! Pure and controlled: the application owns the [`History`] and mirrors [`current`](History::current)
//! into what it shows.
//!
//! ```
//! use iced_adaptive_design::primitives::history::History;
//!
//! let mut h = History::new("Home");
//! h.open("Search");
//! h.open("Album");
//! assert_eq!(h.back(), Some(&"Search"));
//! assert_eq!(h.back(), Some(&"Home"));
//! assert_eq!(h.back(), None);          // nothing earlier: stays where it is
//! ```

/// The current place and the places left behind, most recent last.
#[derive(Debug, Clone, PartialEq)]
pub struct History<P> {
    current: P,
    trail: Vec<P>,
    limit: usize,
}

impl<P> History<P> {
    /// Starts at `place` with no trail. At most 64 places are remembered.
    pub fn new(place: P) -> Self {
        Self {
            current: place,
            trail: Vec::new(),
            limit: 64,
        }
    }
    /// The most places the trail keeps (the oldest are forgotten first).
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }
    /// Where you are.
    pub fn current(&self) -> &P {
        &self.current
    }
    /// Whether Back has somewhere to go.
    pub fn can_back(&self) -> bool {
        !self.trail.is_empty()
    }
    /// The places Back would step through, oldest first.
    pub fn trail(&self) -> &[P] {
        &self.trail
    }
    /// How many places Back can step through.
    pub fn depth(&self) -> usize {
        self.trail.len()
    }
    /// Moves to `place`, remembering the one you leave so Back returns to it.
    pub fn open(&mut self, place: P) {
        let left = std::mem::replace(&mut self.current, place);
        self.trail.push(left);
        if self.trail.len() > self.limit {
            self.trail.remove(0);
        }
    }
    /// Jumps to a top-level `place` and forgets the trail.
    pub fn go(&mut self, place: P) {
        self.current = place;
        self.trail.clear();
    }
    /// Swaps the current place without touching the trail (a page that redirects).
    pub fn replace(&mut self, place: P) {
        self.current = place;
    }
    /// Steps back to the place you came from and returns it, or `None` (staying put) when there
    /// is nowhere earlier.
    pub fn back(&mut self) -> Option<&P> {
        let previous = self.trail.pop()?;
        self.current = previous;
        Some(&self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_returns_to_where_you_came_from() {
        let mut h = History::new("Home");
        h.open("Search");
        h.open("Album");
        assert_eq!(h.back(), Some(&"Search"));
        assert_eq!(*h.current(), "Search");
        assert_eq!(h.back(), Some(&"Home"));
        assert_eq!(h.back(), None);
        assert_eq!(*h.current(), "Home");
    }

    #[test]
    fn it_is_not_a_fixed_parent() {
        // The same album, reached two ways, goes back to two places.
        let mut h = History::new("Home");
        h.open("Artist");
        h.open("Album");
        assert_eq!(h.back(), Some(&"Artist"));
        let mut h = History::new("Home");
        h.open("Search");
        h.open("Album");
        assert_eq!(h.back(), Some(&"Search"));
    }

    #[test]
    fn go_forgets_the_trail() {
        let mut h = History::new("Home");
        h.open("Album");
        h.go("Library");
        assert!(!h.can_back());
        assert_eq!(h.back(), None);
        assert_eq!(*h.current(), "Library");
    }

    #[test]
    fn replace_keeps_the_trail_and_the_limit_forgets_the_oldest() {
        let mut h = History::new(0).limit(2);
        for i in 1..=4 {
            h.open(i);
        }
        assert_eq!(h.depth(), 2);
        h.replace(9);
        assert_eq!(h.back(), Some(&3));
        assert_eq!(h.back(), Some(&2));
        assert_eq!(h.back(), None);
    }
}
