use std::{borrow::Cow, fmt::Display};

#[derive(Debug)]
pub enum RncError {
    Io(std::io::Error),
    Encoding(Cow<'static, str>),
}

impl Display for RncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RncError::Encoding(m) => write!(f, "{}", m),
            RncError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<std::io::Error> for RncError {
    fn from(e: std::io::Error) -> Self {
        RncError::Io(e)
    }
}

impl From<Cow<'static, str>> for RncError {
    fn from(m: Cow<'static, str>) -> Self {
        RncError::Encoding(m)
    }
}
