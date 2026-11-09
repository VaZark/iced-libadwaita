//! Bundled appearance-control SVGs.
use super::{IconSource, IconView};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceIcon {
    Light,
    Dark,
    System,
}
impl AppearanceIcon {
    pub fn source(self) -> &'static str {
        match self {
            Self::Light => include_str!("appearance/light.svg"),
            Self::Dark => include_str!("appearance/dark.svg"),
            Self::System => include_str!("appearance/system.svg"),
        }
    }
    pub fn view<'a>(self, size: f32) -> IconView<'a> {
        IconSource::from(self).view(size)
    }
}
impl From<AppearanceIcon> for IconSource {
    fn from(icon: AppearanceIcon) -> Self {
        Self::svg(icon.source())
    }
}
