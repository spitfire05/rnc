use std::error;
use std::fmt;

const LF: u8 = 0xA;
const CR: u8 = 0xD;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum Conversion {
    Dos2unix,
    Unix2dos,
}

pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

#[derive(Debug, Clone)]
pub struct WrongInputLen {
    input_len: usize,
    char_size: usize,
}

impl fmt::Display for WrongInputLen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl error::Error for WrongInputLen {
    fn description(&self) -> &str {
        "input length was not valid for given char size"
    }

    fn cause(&self) -> Option<&(dyn error::Error)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

fn build_char(c: u8, char_size: usize, order: &ByteOrder) -> Vec<u8> {
    let mut buff: Vec<u8> = Vec::with_capacity(char_size);
    buff.push(c);

    for _ in 1..char_size {
        buff.push(0x0);
    }

    match order {
        ByteOrder::LittleEndian => {
            buff.reverse();
        },
        ByteOrder::BigEndian => {},
    }

    buff
}

pub fn convert(input: &[u8], conv: &Conversion, char_size: usize, order: &ByteOrder) -> Result<Vec<u8>> {
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
    fn erros() {
        convert(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            &Conversion::Dos2unix, 3, &ByteOrder::LittleEndian).unwrap_err();
    }
}
