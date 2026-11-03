//! Visual components: pieces that **draw**, with no behavior and no data logic of their own.
//!
//! A full widget is a composition. The [primitives](crate::primitives) decide *what* (scales,
//! layouts, hit testing); a component here decides *how it looks* (an axis, a legend, a tooltip,
//! a series color); the widget in [`widget`](crate::widget) joins them with the application's
//! data and events. Components take the [`Theme`](crate::Theme) and plain numbers, never
//! application state, so a different widget can reuse them.
//!
//! * [`chart`]: the drawing pieces of the chart widgets.

pub mod chart;
