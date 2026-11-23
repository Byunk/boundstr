use std::error::Error;
use std::fmt;

use crate::encoding::Encoding;

/// Length exceeded error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthExceeded<E> {
    /// The length of the rejected string, in code units in encoding `E`.
    pub len: usize,
    /// The maximum length, in code units in encoding `E`.
    pub max: usize,
    /// The target encoding.
    pub encoding: E,
}

impl<E: Encoding> LengthExceeded<E> {
    /// Creates a new [`LengthExceeded`] error.
    pub(crate) fn new(len: usize, max: usize) -> Self {
        Self {
            len,
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
            "length exceeded: len={} max={} encoding={}",
            self.len, self.max, self.encoding
        )
    }
}
