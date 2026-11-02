//! Validation: when a field shows that it is valid or invalid. Pure and controlled.
//!
//! Mirrors CSS `:user-valid` / `:user-invalid` and Radix `Form.Message`: a field is
//! [`Pristine`](Presentation::Pristine) until the user has interacted with it (it was touched
//! or blurred) or tried to submit; only then does a failing value show as
//! [`Invalid`](Presentation::Invalid) with its message, and a passing one as
//! [`Valid`](Presentation::Valid). The application owns the value and the `touched` flag and
//! passes them in; nothing here stores state.

/// The result of checking a value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Validity {
    #[default]
    Ok,
    /// Acceptable but worth a note (shown, does not block submitting).
    Warning(String),
    /// Not acceptable.
    Error(String),
}

impl Validity {
    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }
    pub fn warning(msg: impl Into<String>) -> Self {
        Self::Warning(msg.into())
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
    /// `true` for [`Error`](Self::Error) only: a warning does not block a submit.
    pub fn blocks_submit(&self) -> bool {
        matches!(self, Self::Error(_))
    }
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Ok => None,
            Self::Warning(m) | Self::Error(m) => Some(m),
        }
    }
}

/// How a field looks. `Pristine` draws nothing special.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Presentation {
    #[default]
    Pristine,
    Valid,
    Warning,
    Invalid,
}

/// What the user has done to a field. `touched`: it was edited or blurred; `submitted`: a
/// submit was attempted on the form it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Interaction {
    pub touched: bool,
    pub submitted: bool,
}

impl Interaction {
    /// Whether validity may be shown yet (`:user-valid` / `:user-invalid` apply).
    pub fn revealed(self) -> bool {
        self.touched || self.submitted
    }
}

/// The presentation of `validity` once `revealed` (see [`Interaction::revealed`]); always
/// `Pristine` before that.
pub fn present(validity: &Validity, revealed: bool) -> Presentation {
    if !revealed {
        return Presentation::Pristine;
    }
    match validity {
        Validity::Ok => Presentation::Valid,
        Validity::Warning(_) => Presentation::Warning,
        Validity::Error(_) => Presentation::Invalid,
    }
}

/// The message to show under the field: the validity's own, only once `revealed`.
pub fn message(validity: &Validity, revealed: bool) -> Option<&str> {
    if revealed { validity.message() } else { None }
}

/// `required`: an empty (or whitespace-only) value is an error with `msg`.
pub fn required(value: &str, msg: &str) -> Validity {
    if value.trim().is_empty() {
        Validity::error(msg)
    } else {
        Validity::Ok
    }
}

/// Runs `validators` over `value` in order: the first error wins, else the first warning,
/// else `Ok`.
pub fn chain(value: &str, validators: &[fn(&str) -> Validity]) -> Validity {
    let mut warning = None;
    for v in validators {
        match v(value) {
            Validity::Ok => {}
            w @ Validity::Warning(_) => {
                warning.get_or_insert(w);
            }
            e @ Validity::Error(_) => return e,
        }
    }
    warning.unwrap_or_default()
}

/// Whether a form may be submitted: none of its fields' validities is an error.
pub fn can_submit<'a>(validities: impl IntoIterator<Item = &'a Validity>) -> bool {
    !validities.into_iter().any(Validity::blocks_submit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn not_empty(s: &str) -> Validity {
        required(s, "Required")
    }
    fn short(s: &str) -> Validity {
        if s.len() < 3 {
            Validity::warning("Short")
        } else {
            Validity::Ok
        }
    }
    fn no_space(s: &str) -> Validity {
        if s.contains(' ') {
            Validity::error("No spaces")
        } else {
            Validity::Ok
        }
    }

    #[test]
    fn pristine_until_interacted() {
        let bad = Validity::error("x");
        assert_eq!(present(&bad, false), Presentation::Pristine);
        assert_eq!(message(&bad, false), None);
        assert_eq!(present(&bad, true), Presentation::Invalid);
        assert_eq!(message(&bad, true), Some("x"));
        assert_eq!(present(&Validity::Ok, true), Presentation::Valid);
        assert_eq!(
            present(&Validity::warning("w"), true),
            Presentation::Warning
        );
    }

    #[test]
    fn touched_or_submitted_reveals() {
        assert!(!Interaction::default().revealed());
        assert!(
            Interaction {
                touched: true,
                submitted: false
            }
            .revealed()
        );
        assert!(
            Interaction {
                touched: false,
                submitted: true
            }
            .revealed()
        );
    }

    #[test]
    fn required_trims() {
        assert!(required("  ", "r").blocks_submit());
        assert!(required("a", "r").is_ok());
    }

    #[test]
    fn chain_prefers_errors_then_first_warning() {
        let all = [not_empty, short, no_space];
        assert_eq!(chain("", &all), Validity::error("Required"));
        assert_eq!(chain("a b", &all), Validity::error("No spaces"));
        assert_eq!(chain("ab", &all), Validity::warning("Short"));
        assert_eq!(chain("abcd", &all), Validity::Ok);
    }

    #[test]
    fn warnings_do_not_block_submit() {
        assert!(can_submit(&[Validity::Ok, Validity::warning("w")]));
        assert!(!can_submit(&[Validity::Ok, Validity::error("e")]));
    }
}
