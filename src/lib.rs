//! A library for newline character converting
//!
//! The main struct of this crate is [`Converter`] which can be used to configure and run the newline conversion.
//!
//! [`Converter`]: struct.Converter.html

mod converter;
mod errors;
mod utils;

pub use crate::converter::Converter;
pub use crate::utils::{ByteOrder, Conversion};
