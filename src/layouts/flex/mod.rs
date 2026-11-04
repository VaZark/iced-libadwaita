//! Flex sizing along one line, as CSS `flex` does it: the pure [`rule`] and the widgets that apply it.

pub(crate) mod cells;
pub(crate) mod row;
pub mod rule;

pub use cells::{Flex, FlexCells};
pub use row::FlexRow;
