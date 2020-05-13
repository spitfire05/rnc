use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct WrongInputLen {
    pub input_len: usize,
    pub char_size: usize,
}

impl fmt::Display for WrongInputLen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "input length was not valid for given char size")
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

#[derive(Debug, Clone)]
pub struct CharSizeZero;

impl fmt::Display for CharSizeZero {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Character size has to be non-zero")
    }
}

impl error::Error for CharSizeZero {
    fn description(&self) -> &str {
        "Character size has to be non-zero"
    }

    fn cause(&self) -> Option<&(dyn error::Error)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}