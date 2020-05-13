//! A library for newline character converting

mod errors;
mod converter;
mod utils;

pub use crate::converter::*;
pub use crate::utils::{ByteOrder, Conversion};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_conversion() {
        assert_eq!(convert(b"foo\r\nbar", &Conversion::Dos2unix, 1,&ByteOrder::LittleEndian).unwrap(), b"foo\nbar");
        assert_eq!(convert(b"foo\nbar", &Conversion::Unix2dos, 1, &ByteOrder::LittleEndian).unwrap(), b"foo\r\nbar");
    }

    #[test]
    fn advanced() {
        assert_eq!(convert("foo\r\nbar".as_bytes(), &Conversion::Unix2dos, 1, &ByteOrder::LittleEndian).unwrap(), "foo\r\nbar".as_bytes());
        assert_eq!(convert("foo\r\nbar".as_bytes(), &Conversion::Unix2dos, 1, &ByteOrder::LittleEndian).unwrap(), "foo\r\nbar".as_bytes());
        assert_eq!(convert("foo\rbar\r\n".as_bytes(), &Conversion::Dos2unix, 1, &ByteOrder::LittleEndian).unwrap(), "foo\rbar\n".as_bytes());
        let utf16_le_dos: [u8; 8] = [0x00, 0x42, 0x00, 0x0D, 0x00, 0x0A, 0x00, 0x41];
        let utf16_le_unix: [u8; 6] = [0x00, 0x42, 0x00, 0x0A, 0x00, 0x41];
        assert_eq!(convert(&utf16_le_dos, &Conversion::Unix2dos, 2, &ByteOrder::LittleEndian).unwrap(), utf16_le_dos);
        assert_eq!(convert(&utf16_le_dos, &Conversion::Dos2unix, 2, &ByteOrder::LittleEndian).unwrap(), utf16_le_unix);
    }

    #[test]
    fn errors() {
        convert(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            &Conversion::Dos2unix, 3, &ByteOrder::LittleEndian).unwrap_err();
        convert(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            &Conversion::Dos2unix, 0, &ByteOrder::LittleEndian).unwrap_err();
    }
}
