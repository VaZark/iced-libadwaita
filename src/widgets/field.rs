//! Shared input presentation, implemented at the component layer.
pub use crate::components::{
    InputWrapper as Field, InputWrapperLayout as FieldLayout, input_outline as outlined,
};

/// Reusable presentation for the underlying input wrapper.
pub use crate::components::input_wrapper::InputWrapperProps as FieldProps;
