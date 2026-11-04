//! Metas: components that deal with whole widgets. [`MetaComponent`] chooses one of several widgets by a trigger;
//! [`ComposedComponent`] shows several together.

pub(crate) mod composed_component;
pub(crate) mod meta_component;
pub(crate) mod meta_parent_component;

pub use composed_component::{ComposeAxis, ComposedComponent};
pub use meta_component::MetaComponent;
pub use meta_parent_component::{MetaParentComponent, Reporter};
