//! Flex sizing along one line, as CSS `flex` does it: each item has a **basis** (its preferred
//! size), a **grow** factor, a **shrink** factor and a **minimum**. Free space goes to the items
//! that grow, in proportion; a shortage is taken from the items that shrink, in proportion to
//! their basis, but never below an item's minimum.
//!
//! It is what a row of columns needs so that every column gives way together and none is cut
//! off: a small column keeps what its label needs (its minimum) while a big one shrinks, and the
//! header and the rows, resolving the same items, line up exactly.

/// One item on the line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    /// The preferred size.
    pub basis: f32,
    /// A share of the free space when there is some (0 does not grow).
    pub grow: f32,
    /// A share of the shortage when there is one (0 does not shrink).
    pub shrink: f32,
    /// The least size, however short of room the line is.
    pub min: f32,
}

impl Item {
    /// Keeps its size: a picture, a menu button.
    pub fn fixed(size: f32) -> Self {
        Self {
            basis: size,
            grow: 0.0,
            shrink: 0.0,
            min: size,
        }
    }
    /// Starts at `basis`, shrinks down to `min`, and does not grow.
    pub fn shrinking(basis: f32, min: f32) -> Self {
        Self {
            basis,
            grow: 0.0,
            shrink: 1.0,
            min: min.min(basis),
        }
    }
    /// Starts at `basis`, takes `grow` shares of any free space, shrinks down to `min`.
    pub fn flex(basis: f32, grow: f32, min: f32) -> Self {
        Self {
            basis,
            grow,
            shrink: 1.0,
            min: min.min(basis),
        }
    }
}

/// The size of each item on a line of `available` with `gap` between items.
///
/// The sizes plus the gaps fill the line exactly when something can grow (or the items had to
/// shrink); with nothing to grow, a line with room to spare leaves it empty. When the minimums
/// do not fit either, the items are at their minimums and the line overflows.
pub fn resolve(items: &[Item], available: f32, gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let room = (available - gap * (n as f32 - 1.0)).max(0.0);
    let mut size: Vec<f32> = items.iter().map(|i| i.basis.max(i.min)).collect();
    let total: f32 = size.iter().sum();

    if total < room {
        // Free space: shared by `grow`.
        let grow: f32 = items.iter().map(|i| i.grow).sum();
        if grow > 0.0 {
            let extra = room - total;
            for (s, i) in size.iter_mut().zip(items) {
                *s += extra * i.grow / grow;
            }
        }
    } else if total > room {
        // A shortage: taken in proportion to basis, items that hit their minimum stop giving
        // and the rest take what is left, until it is all found or nothing more can give.
        let mut frozen = vec![false; n];
        for _ in 0..n {
            let used: f32 = size.iter().sum();
            let short = used - room;
            if short <= 1e-4 {
                break;
            }
            let weight: f32 = (0..n)
                .filter(|&k| !frozen[k] && items[k].shrink > 0.0)
                .map(|k| items[k].basis * items[k].shrink)
                .sum();
            if weight <= 0.0 {
                break;
            }
            let mut hit = false;
            for k in 0..n {
                if frozen[k] || items[k].shrink <= 0.0 {
                    continue;
                }
                let take = short * items[k].basis * items[k].shrink / weight;
                if size[k] - take <= items[k].min {
                    size[k] = items[k].min;
                    frozen[k] = true;
                    hit = true;
                } else {
                    size[k] -= take;
                }
            }
            if !hit {
                break;
            }
        }
    }
    size
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sum(v: &[f32]) -> f32 {
        v.iter().sum()
    }

    #[test]
    fn free_space_goes_to_the_items_that_grow() {
        let items = [
            Item::fixed(40.0),
            Item::flex(100.0, 1.0, 50.0),
            Item::flex(100.0, 3.0, 50.0),
        ];
        let w = resolve(&items, 540.0, 0.0);
        assert_eq!(w[0], 40.0);
        // 300 free, split 1:3.
        assert_eq!((w[1], w[2]), (175.0, 325.0));
        assert!((sum(&w) - 540.0).abs() < 1e-3);
    }

    #[test]
    fn a_shortage_is_shared_in_proportion_to_basis() {
        let items = [Item::shrinking(300.0, 0.0), Item::shrinking(100.0, 0.0)];
        let w = resolve(&items, 200.0, 0.0);
        assert!((w[0] - 150.0).abs() < 1e-3 && (w[1] - 50.0).abs() < 1e-3);
    }

    #[test]
    fn a_small_item_stops_at_its_minimum_and_the_others_give_more() {
        // The 44 px column cannot go below 44: the title and album give the rest.
        let items = [
            Item::flex(120.0, 1.0, 50.0),
            Item::shrinking(160.0, 40.0),
            Item::shrinking(44.0, 44.0),
        ];
        let w = resolve(&items, 200.0, 0.0);
        assert_eq!(w[2], 44.0);
        assert!(w[0] >= 50.0 && w[1] >= 40.0);
        assert!((sum(&w) - 200.0).abs() < 1e-2);
    }

    #[test]
    fn fixed_items_never_shrink_and_the_line_overflows_when_minimums_do_not_fit() {
        let items = [
            Item::fixed(40.0),
            Item::flex(100.0, 1.0, 60.0),
            Item::fixed(36.0),
        ];
        let w = resolve(&items, 100.0, 0.0);
        assert_eq!((w[0], w[2]), (40.0, 36.0));
        // 100 - 76 = 24 is below the 60 minimum: it holds, and the line overflows.
        assert_eq!(w[1], 60.0);
    }

    #[test]
    fn gaps_come_out_of_the_room() {
        let items = [Item::flex(0.0, 1.0, 0.0), Item::flex(0.0, 1.0, 0.0)];
        let w = resolve(&items, 110.0, 10.0);
        assert_eq!(w, vec![50.0, 50.0]);
    }

    #[test]
    fn nothing_that_grows_leaves_the_room_empty_and_empty_is_empty() {
        let w = resolve(&[Item::fixed(10.0), Item::shrinking(20.0, 0.0)], 100.0, 0.0);
        assert_eq!(w, vec![10.0, 20.0]);
        assert!(resolve(&[], 100.0, 0.0).is_empty());
    }

    #[test]
    fn the_title_is_never_squeezed_out() {
        // A narrow row: title (flex), album, time, a picture and a menu.
        let items = [
            Item::fixed(40.0),
            Item::flex(120.0, 1.0, 51.0),
            Item::shrinking(160.0, 80.0),
            Item::shrinking(44.0, 44.0),
            Item::fixed(36.0),
        ];
        let w = resolve(&items, 380.0, 12.0);
        assert!(w[1] >= 51.0, "the title keeps at least its label: {w:?}");
        assert!(w[3] >= 44.0, "the time keeps its label: {w:?}");
    }
}
