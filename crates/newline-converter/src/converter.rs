use std::{error, iter::Peekable, slice::Chunks};

use crate::errors::*;
use crate::utils::*;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct Converter {
    char_size: usize,
    conversion: Conversion,
    cr: Vec<u8>,
    lf: Vec<u8>,
}

impl Converter {
    /// Returns Converter with customized properties
    ///
    /// # Arguments
    /// * `conv` - type of conversion to perform
    /// * `char_size` - how many bytes to interpret as a character. Should be `1` for UTF8, `2` for UTF16 etc.
    /// * `order` - Byte order. This value does not matter if `char_size == 1`
    pub fn new(conversion: Conversion, char_size: usize, byte_order: ByteOrder) -> Converter {
        let cr: Vec<u8> = build_char(CR, char_size, &byte_order);
        let lf: Vec<u8> = build_char(LF, char_size, &byte_order);

        Converter {
            char_size,
            conversion,
            cr,
            lf,
        }
    }

    /// Returns UTF8-configured Converter
    ///
    /// # Examples
    /// ```
    /// use newline_converter::*;
    /// let converter = Converter::utf8(Conversion::Dos2unix);
    /// assert_eq!(converter.convert(b"foo\r\nbar").unwrap(), b"foo\nbar");
    /// ```
    ///
    /// Use `&str.bytes()` and `String::from_utf8` to deal with Rust strings:
    /// ```
    /// use newline_converter::*;
    /// let someString = String::from("foobar\r\n");
    /// let converter = Converter::utf8(Conversion::Dos2unix);
    /// let bytes = converter.convert(someString.as_bytes()).unwrap();
    /// let convertedString = String::from_utf8(bytes).unwrap();
    /// assert_eq!("foobar\n", convertedString);
    pub fn utf8(conversion: Conversion) -> Converter {
        Converter::new(conversion, 1, ByteOrder::BigEndian)
    }

    /// Returns UTF16LE-configured Converter
    ///
    /// # Examples
    /// ```
    /// use newline_converter::*;
    /// let converter = Converter::utf16le(Conversion::Dos2unix);
    /// assert_eq!(converter.convert(b"\0f\0o\0o\0\r\0\n\0b\0a\0r").unwrap(), b"\0f\0o\0o\0\n\0b\0a\0r");
    /// ```
    pub fn utf16le(conversion: Conversion) -> Converter {
        Converter::new(conversion, 2, ByteOrder::LittleEndian)
    }

    /// Returns UTF16BE-configured Converter
    ///
    /// # Examples
    /// ```
    /// use newline_converter::*;
    /// let converter = Converter::utf16be(Conversion::Dos2unix);
    /// assert_eq!(converter.convert(b"f\0o\0o\0\r\0\n\0b\0a\0r\0").unwrap(), b"f\0o\0o\0\n\0b\0a\0r\0");
    /// ```
    pub fn utf16be(conversion: Conversion) -> Converter {
        Converter::new(conversion, 2, ByteOrder::BigEndian)
    }

    /// Returns UTF32LE-configured Converter
    ///
    /// # Examples
    /// ```
    /// use newline_converter::*;
    /// let converter = Converter::utf32le(Conversion::Dos2unix);
    /// assert_eq!(converter.convert(b"\0\0\0f\0\0\0o\0\0\0o\0\0\0\r\0\0\0\n\0\0\0b\0\0\0a\0\0\0r").unwrap(), b"\0\0\0f\0\0\0o\0\0\0o\0\0\0\n\0\0\0b\0\0\0a\0\0\0r");
    /// ```
    pub fn utf32le(conversion: Conversion) -> Converter {
        Converter::new(conversion, 4, ByteOrder::LittleEndian)
    }

    /// Returns UTF32BE-configured Converter
    ///
    /// # Examples
    /// ```
    /// use newline_converter::*;
    /// let converter = Converter::utf32be(Conversion::Dos2unix);
    /// assert_eq!(converter.convert(b"f\0\0\0o\0\0\0o\0\0\0\r\0\0\0\n\0\0\0b\0\0\0a\0\0\0r\0\0\0").unwrap(), b"f\0\0\0o\0\0\0o\0\0\0\n\0\0\0b\0\0\0a\0\0\0r\0\0\0");
    /// ```
    pub fn utf32be(conversion: Conversion) -> Converter {
        Converter::new(conversion, 4, ByteOrder::BigEndian)
    }

    /// Returns a new buffer with converted newline
    ///
    ///
    /// # Examples
    /// ```
    /// use newline_converter::*;
    /// let converter = Converter::new(Conversion::Dos2unix, 1, ByteOrder::LittleEndian);
    /// assert_eq!(converter.convert(b"foo\r\nbar").unwrap(), b"foo\nbar");
    /// ```
    pub fn convert(&self, input: &[u8]) -> Result<Vec<u8>> {
        if self.char_size == 0 {
            return Err(Box::new(CharSizeZero));
        }

        if input.len() % self.char_size != 0 {
            return Err(Box::new(WrongInputLen {
                input_len: input.len(),
                char_size: self.char_size,
            }));
        }

        let mut output: Vec<u8> = match self.conversion {
            // crude size guessing
            Conversion::Dos2unix => {
                let n = input.iter().filter(|x| **x == CR).count();
                Vec::with_capacity(input.len() - (n * self.char_size))
            }
            Conversion::Unix2dos => {
                let n = input.iter().filter(|x| **x == LF).count();
                Vec::with_capacity(input.len() + (n * self.char_size))
            }
        };

        match self.conversion {
            Conversion::Dos2unix => self.dos2unix(input.chunks(self.char_size).peekable(), &mut output),
            Conversion::Unix2dos => self.unix2dos(input.chunks(self.char_size), &mut output),
        }

        Ok(output)
    }

    fn dos2unix(&self, mut iter: Peekable<Chunks<u8>>, output: &mut Vec<u8>) {
        let empty: &[u8] = &[];
        while let Some(current) = iter.next() {
            if self.cr == current {
                let next = *iter.peek().unwrap_or(&empty);
                if self.lf == next {
                    // drop it
                    continue;
                }
            }
            output.extend(current);
        }
    }

    fn unix2dos(&self, iter: Chunks<u8>, output: &mut Vec<u8>)
    {
        let mut last_char: Option<&[u8]> = None;
        for buffer in iter {
            if self.lf == buffer && last_char.is_some() && self.cr != last_char.unwrap() {
                output.extend(self.cr.clone());
            }
            last_char = Some(buffer);
            output.extend(buffer);
        }
    }
}
