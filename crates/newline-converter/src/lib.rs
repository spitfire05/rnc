//! A library for newline character converting.
//!
//! This crate provides two functions: [`dos2unix`] and [`unix2dos`] that perform the conversion on strings.
//!
//! The conversion functions are **lazy** - they don't perform any allocations if the input is already in correct format.
//!
//! [`dos2unix`]: fn.dos2unix.html
//! [`unix2dos`]: fn.unix2dos.html

use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;

#[deny(clippy::unwrap_used)]
#[deny(clippy::expect_used)]

const UNPACK_MSG: &str = "Grapheme should always be found -- Please file a bug report";

macro_rules! unpack_grapheme {
    ($x:expr) => {
        match $x {
            Some(i) => i,
            None => unreachable!("{}", UNPACK_MSG),
        }
    };
}

/// Converts DOS-style line endings (`\r\n`) to UNIX-style (`\n`).
///
/// The input string may already be in correct format, so this function
/// returns `Cow<str>`, to avoid unnecessary allocation and copying.
///
/// # Examples
/// ```
/// assert_eq!(newline_converter::dos2unix("\r\nfoo\r\nbar\r\n"), "\nfoo\nbar\n");
/// ```
///
/// Lone `\r` bytes will be preserved:
/// ```
///  assert_eq!(
///    newline_converter::dos2unix("\nfoo\rbar\r\n"),
///    "\nfoo\rbar\n"
///  );
/// ```
pub fn dos2unix<T: AsRef<str> + ?Sized>(input: &T) -> Cow<str> {
    let mut iter = input.as_ref().chars().peekable();

    let input = input.as_ref();
    let mut output: Option<String> = None;

    while let Some(current) = iter.next() {
        if '\r' == current {
            if let Some('\n') = iter.peek() {
                // drop it
                if output.is_none() {
                    let n = input.chars().filter(|x| *x == '\r').count();
                    let mut buffer = String::with_capacity(input.len() - n);
                    let i = unpack_grapheme!(input
                        .grapheme_indices(true)
                        .find(|(_, x)| *x == "\r\n")
                        .map(|(i, _)| i));
                    let (past, _) = input.split_at(i);
                    buffer.push_str(past);
                    output = Some(buffer);
                }
                continue;
            }
        }
        if output.is_some() {
            output.as_mut().unwrap().push(current);
        }
    }

    match output {
        None => Cow::Borrowed(input),
        Some(o) => Cow::Owned(o),
    }
}

#[allow(clippy::match_like_matches_macro)] // MSRV 1.38, matches! macro available in 1.42
/// Converts UNIX-style line endings (`\n`) to DOS-style (`\r\n`).
///
/// The input string may already be in correct format, so this function
/// returns `Cow<str>`, to avoid unnecessary allocation and copying.
///
/// # Examples
/// ```
/// assert_eq!(newline_converter::unix2dos("\nfoo\nbar\n"), "\r\nfoo\r\nbar\r\n");
/// ```
///
/// Already present DOS line breaks are respected:
/// ```
/// assert_eq!(newline_converter::unix2dos("\nfoo\r\nbar\n"), "\r\nfoo\r\nbar\r\n");
/// ```
pub fn unix2dos<T: AsRef<str> + ?Sized>(input: &T) -> Cow<str> {
    let mut output: Option<String> = None;
    let mut last_char: Option<char> = None;

    let input = input.as_ref();
    for (i, current) in input.chars().enumerate() {
        if '\n' == current
            && (i == 0
                || match last_char {
                    Some('\r') => false,
                    _ => true,
                })
        {
            if output.is_none() {
                let n = input.chars().filter(|x| *x == '\n').count();
                let mut buffer = String::with_capacity(input.len() + n);
                let i = unpack_grapheme!(input
                    .grapheme_indices(true)
                    .find(|(_, x)| *x == "\n")
                    .map(|(i, _)| i));
                let (past, _) = input.split_at(i);
                buffer.push_str(past);
                output = Some(buffer);
            }
            output.as_mut().unwrap().push('\r');
        }
        last_char = Some(current);

        if let Some(o) = output.as_mut() {
            o.push(current);
        }
    }

    match output {
        Some(o) => Cow::Owned(o),
        None => Cow::Borrowed(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn middle() {
        assert_eq!(dos2unix("foo\r\nbar"), "foo\nbar");
        assert_eq!(unix2dos("foo\nbar"), "foo\r\nbar");
    }

    #[test]
    fn beginning() {
        assert_eq!(dos2unix("\r\nfoobar"), "\nfoobar");
        assert_eq!(unix2dos("\nfoobar"), "\r\nfoobar");
    }

    #[test]
    fn end() {
        assert_eq!(dos2unix("foobar\r\n"), "foobar\n");
        assert_eq!(unix2dos("foobar\n"), "foobar\r\n");
    }

    #[test]
    fn all() {
        assert_eq!(dos2unix("\r\nfoo\r\nbar\r\n"), "\nfoo\nbar\n");
        assert_eq!(unix2dos("\nfoo\nbar\n"), "\r\nfoo\r\nbar\r\n");
    }

    #[test]
    fn advanced() {
        assert_eq!(unix2dos("\rfoo\r\nbar\n"), "\rfoo\r\nbar\r\n");
        assert_eq!(dos2unix("\nfoo\rbar\r\n"), "\nfoo\rbar\n");
    }

    #[test]
    fn not_mutated_dos2unix() {
        let converted = dos2unix("\nfoo\nbar\n");
        assert_eq!(converted, Cow::Borrowed("\nfoo\nbar\n") as Cow<str>);
    }

    #[test]
    fn mutated_dos2unix() {
        let converted = dos2unix("\r\nfoo\r\nbar\r\n");
        assert_eq!(
            converted,
            Cow::Owned(String::from("\nfoo\nbar\n")) as Cow<str>
        );
    }

    #[test]
    fn not_mutated_unix2dos() {
        let converted = unix2dos("\r\nfoo\r\nbar\r\n");
        assert_eq!(converted, Cow::Borrowed("\r\nfoo\r\nbar\r\n") as Cow<str>);
    }

    #[test]
    fn mutated_unix2dos() {
        let converted = unix2dos("\nfoo\nbar\n");
        assert_eq!(
            converted,
            Cow::Owned(String::from("\r\nfoo\r\nbar\r\n")) as Cow<str>
        );
    }

    #[test]
    fn non_ascii_characters_unix2dos() {
        assert_eq!(
            unix2dos("Zażółć\ngęślą\njaźń\n"),
            "Zażółć\r\ngęślą\r\njaźń\r\n"
        );
    }

    #[test]
    fn non_ascii_characters_dos2unix() {
        assert_eq!(
            dos2unix("Zażółć\r\ngęślą\r\njaźń\r\n"),
            "Zażółć\ngęślą\njaźń\n"
        );
    }

    #[test]
    // https://github.com/spitfire05/rnc/issues/14
    fn panics_in_0_2_1_unix2dos() {
        assert_eq!(unix2dos("ä\n"), "ä\r\n");
    }

    #[test]
    // https://github.com/spitfire05/rnc/issues/14
    fn panics_in_0_2_1_dos2unix() {
        assert_eq!(dos2unix("ä\r\n"), "ä\n");
    }
}
