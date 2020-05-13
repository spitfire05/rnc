use std::error;

use crate::utils::*;
use crate::errors::*;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Performs newline byte(s) conversion on given buffer
/// 
/// # Arguments
/// * `input` - the input buffer to convert
/// * `conv` - type of conversion to perform
/// * `char_size` - how many bytes to interpret as a character. Should be `1` for UTF8, `2` for UTF16 etc.
/// * `order` - Byte order. This value does not matter if `char_size == 1`
///
/// # Examples
/// ```
/// use rnc::{Conversion, ByteOrder, convert};
/// assert_eq!(convert(b"foo\r\nbar", &Conversion::Dos2unix, 1,&ByteOrder::LittleEndian).unwrap(), b"foo\nbar");
/// assert_eq!(convert(b"foo\nbar", &Conversion::Unix2dos, 1, &ByteOrder::LittleEndian).unwrap(), b"foo\r\nbar");
/// ```
pub fn convert(input: &[u8], conv: &Conversion, char_size: usize, order: &ByteOrder) -> Result<Vec<u8>> {
    if char_size == 0 {
        return Err(Box::new(CharSizeZero));
    }

    if input.len() % char_size != 0 {
        return Err(Box::new(WrongInputLen {
            input_len: input.len(),
            char_size: char_size,
        }))
    }

    let cr: Vec<u8> = build_char(CR, char_size, &order);
    let lf: Vec<u8> = build_char(LF, char_size, &order);

    let mut i = 0;
    let mut output: Vec<u8> = match conv {
        // crude size guessing
        Conversion::Dos2unix => {
            let n = input.iter().filter(|x| **x == CR).count();
            Vec::with_capacity(input.len() - (n * char_size))
        },
        Conversion::Unix2dos => {
            let n = input.iter().filter(|x| **x == LF).count();
            Vec::with_capacity(input.len() + (n * char_size))
        }
    };
    let mut last_char: Option<&[u8]> = None;
    let mut lookahead: Vec<u8> = Vec::with_capacity(char_size);

    while i < input.len() {
        let (left, _) = input.split_at(i + char_size);
        let (_, buffer) = left.split_at(i);
        debug_assert_eq!(buffer.len(), char_size);
        match conv {
            Conversion::Dos2unix => {
                if cr == buffer {
                    if i + (char_size * 2) <= input.len()
                    {
                        lookahead.clear();
                        for x in i + char_size..i + (char_size * 2) {
                            lookahead.push(input[x]);
                        }
                        if lookahead == lf {
                            // drop it
                        }
                        else {
                            output.extend(buffer);
                        }
                    }
                    else {
                        // this is the last character, let it be
                        output.extend(buffer);
                    }
                }
                else {
                    output.extend(buffer);
                }
            },
            Conversion::Unix2dos => {
                if lf == buffer {
                    // check if we are not multiplying CRs here
                    if last_char.is_some() && cr != last_char.unwrap() {
                        output.extend(cr.clone());
                    }
                    last_char = Some(buffer);
                    output.extend(buffer);
                }
                else {
                    last_char = Some(buffer);
                    output.extend(buffer);
                }
            }
        }

        i += char_size;
    }

    Ok(output)

}