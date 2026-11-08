//! Controlled application appearance preference and cycling rules.
use crate::Scheme;

/// Application appearance preference. System is the default and clears the app override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}
impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }
    pub fn scheme_override(self) -> Option<Scheme> {
        match self {
            Self::System => None,
            Self::Light => Some(Scheme::Light),
            Self::Dark => Some(Scheme::Dark),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}
impl From<Option<Scheme>> for ThemeMode {
    fn from(scheme: Option<Scheme>) -> Self {
        match scheme {
            None => Self::System,
            Some(Scheme::Light) => Self::Light,
            Some(Scheme::Dark) => Self::Dark,
        }
    }
}
