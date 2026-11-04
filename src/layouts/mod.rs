//! Layouts: where things go and how they flow. Grid, flex, stack, overlay, dock, split, clip, virtualization and motion.
//!
//! No domain meaning and no behavior of their own beyond placement; a layout's pure geometry is its
//! `rule` (or a plain module such as [`anchor`]), unit-tested without a window.

pub mod anchor;
pub(crate) mod anchored;
pub mod animation;
pub(crate) mod clip_window;
pub(crate) mod container;
pub(crate) mod content_area;
pub mod dock;
pub mod flex;
pub(crate) mod flow_grid;
pub(crate) mod motion;
pub mod overlay;
pub(crate) mod scrolled;
pub(crate) mod slots;
pub(crate) mod split_view;
pub mod splitter;
pub mod surface;
pub(crate) mod table_grid;
pub mod transition;
pub(crate) mod virtual_list;
pub mod virtualize;

pub use anchored::{Anchored, Indicator};
pub use clip_window::{Anchor as ClipAnchor, ClipWindow};
pub use container::{Container, ContainerSize};
pub use content_area::{ContentArea, content_area};
pub use dock::{Dock, DockMode, DockScope, DockSize};
pub use flex::FlexRow;
pub use flex::{Flex, FlexCells};
pub use flow_grid::FlowGrid;
pub use motion::{
    DrawerLayer, Entrance, Highlight, PageTransition, PushStack, Reveal, drawer_layer, entrance,
    highlight, layered, page_transition, push_stack, reveal,
};
pub use overlay::{
    Layer as OverlayLayer, Overlay, closed as overlay_closed, stacked as overlay_stack,
};
pub use scrolled::{scrollbar, scrolled_window, scrolled_window_both, scrolled_window_horizontal};
pub use split_view::{AdaptiveSplit, adaptive_split};
pub use splitter::{Splitter, SplitterOrientation};
pub use surface::window;
pub use table_grid::{TableCell, TableGrid};
pub use virtual_list::VirtualList;
