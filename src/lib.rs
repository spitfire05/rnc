//! A library for newline character converting
//!
//! The main struct of this crate is `Converter` which can be used to configure and run the newline conversion.

mod errors;
mod converter;
mod utils;

pub use crate::converter::Converter;
pub use crate::utils::{ByteOrder, Conversion};
