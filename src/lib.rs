//! A library for newline character converting
//!
//! The main struct of this crate is `Converter` which can be used to configure and run the newline conversion.

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
        let dos2unix = Converter::utf8(Conversion::Dos2unix);
        let unix2dos = Converter::utf8(Conversion::Unix2dos);
        assert_eq!(dos2unix.convert(b"foo\r\nbar").unwrap(), b"foo\nbar");
        assert_eq!(unix2dos.convert(b"foo\nbar").unwrap(), b"foo\r\nbar");
    }

    #[test]
    fn advanced() {
        let dos2unix = Converter::utf8(Conversion::Dos2unix);
        let unix2dos = Converter::utf8(Conversion::Unix2dos);
        assert_eq!(unix2dos.convert(b"\rfoo\r\nbar\n").unwrap(), b"\rfoo\r\nbar\r\n");
        assert_eq!(dos2unix.convert(b"\nfoo\rbar\r\n").unwrap(), b"\nfoo\rbar\n");
        let utf16_le_dos: [u8; 8] = [0x00, 0x42, 0x00, 0x0D, 0x00, 0x0A, 0x00, 0x41];
        let utf16_le_unix: [u8; 6] = [0x00, 0x42, 0x00, 0x0A, 0x00, 0x41];
        let dos2unix_utf16le = Converter::utf16le(Conversion::Dos2unix);
        let unix2dos_utf16le = Converter::utf16le(Conversion::Unix2dos);
        assert_eq!(unix2dos_utf16le.convert(&utf16_le_dos).unwrap(), utf16_le_dos);
        assert_eq!(dos2unix_utf16le.convert(&utf16_le_dos).unwrap(), utf16_le_unix);
    }

    #[test]
    fn errors() {
        let converter_3 = Converter::new(Conversion::Dos2unix, 3, ByteOrder::LittleEndian);
        let converter_0 = Converter::new(Conversion::Dos2unix, 3, ByteOrder::LittleEndian);
        converter_3.convert(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap_err();
        converter_0.convert(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap_err();
    }
}
