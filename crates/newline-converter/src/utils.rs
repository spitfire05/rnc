pub const LF: u8 = 0xA;
pub const CR: u8 = 0xD;
/// Defines conversion operation
#[derive(Copy, Clone)]
pub enum Conversion {
    /// Replaces `\r\n` with `\n`
    Dos2unix,
    /// Replaces `\n` with `\r\n`
    Unix2dos,
}

/// Represents the byte ordering in processed data.
#[derive(Copy, Clone)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

pub fn build_char(c: u8, char_size: usize, order: &ByteOrder) -> Vec<u8> {
    let mut buff: Vec<u8> = Vec::with_capacity(char_size);
    buff.push(c);

    for _ in 1..char_size {
        buff.push(0x0);
    }

    match order {
        ByteOrder::LittleEndian => {
            buff.reverse();
        }
        ByteOrder::BigEndian => {}
    }

    buff
}
