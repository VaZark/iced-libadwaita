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
            Self::Light => iad_icons::appearance::LIGHT,
            Self::Dark => iad_icons::appearance::DARK,
            Self::System => iad_icons::appearance::SYSTEM,
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
