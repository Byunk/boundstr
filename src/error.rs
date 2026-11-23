use std::error::Error;
use std::fmt;

use crate::encoding::Encoding;

/// Length exceeded error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthExceeded<E> {
    /// The maximum length, in code units in encoding `E`.
    pub max: usize,
    /// The target encoding.
    pub encoding: E,
}

impl<E: Encoding> LengthExceeded<E> {
    /// Creates a new [`LengthExceeded`] error.
    pub(crate) fn new(max: usize) -> Self {
        Self {
            max,
            encoding: E::default(),
        }
    }
}

impl<E: Encoding> Error for LengthExceeded<E> {}

impl<E: Encoding> fmt::Display for LengthExceeded<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "length exceeded: max={} encoding={}",
            self.max, self.encoding
        )
    }
}

/// A possible error when converting a UTF-8 byte vector to a bounded string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromUtf8Error<E> {
    InvalidUtf8(std::string::FromUtf8Error),
    LengthExceeded(LengthExceeded<E>),
}

impl<E: Encoding + 'static> Error for FromUtf8Error<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidUtf8(e) => Some(e),
            Self::LengthExceeded(e) => Some(e),
        }
    }
}

impl<E: Encoding> fmt::Display for FromUtf8Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8(error) => fmt::Display::fmt(error, f),
            Self::LengthExceeded(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl<E> From<std::string::FromUtf8Error> for FromUtf8Error<E> {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::InvalidUtf8(e)
    }
}

impl<E> From<LengthExceeded<E>> for FromUtf8Error<E> {
    fn from(e: LengthExceeded<E>) -> Self {
        Self::LengthExceeded(e)
    }
}

/// A possible error when converting a UTF-16 slice to a bounded string.
#[derive(Debug)]
pub enum FromUtf16Error<E> {
    InvalidUtf16(std::string::FromUtf16Error),
    LengthExceeded(LengthExceeded<E>),
}

impl<E: Encoding + 'static> Error for FromUtf16Error<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidUtf16(e) => Some(e),
            Self::LengthExceeded(e) => Some(e),
        }
    }
}

impl<E: Encoding> fmt::Display for FromUtf16Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf16(error) => fmt::Display::fmt(error, f),
            Self::LengthExceeded(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl<E> From<std::string::FromUtf16Error> for FromUtf16Error<E> {
    fn from(e: std::string::FromUtf16Error) -> Self {
        Self::InvalidUtf16(e)
    }
}

impl<E> From<LengthExceeded<E>> for FromUtf16Error<E> {
    fn from(e: LengthExceeded<E>) -> Self {
        Self::LengthExceeded(e)
    }
}
