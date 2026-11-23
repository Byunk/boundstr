use std::fmt;

/// A character encoding.
pub trait Encoding: Sized + Default + fmt::Debug + fmt::Display {
    /// Returns the length of `s` in this encoding.
    fn len(s: &str) -> usize;

    /// Returns the length of `ch` in this encoding.
    fn len_char(ch: char) -> usize;

    /// Finds the closest character boundary `x` not exceeding `index`.
    fn floor_char_boundary(s: &str, index: usize) -> usize;

    /// Returns `true` if the length of `s` is less than or equal to `max` in
    /// this encoding.
    ///
    /// This is derived from `Self::len(s) <= max` by default, but some
    /// encodings may have a faster implementation.
    #[inline]
    fn fits(s: &str, max: usize) -> bool {
        s.len() <= max
    }
}

/// UTF-8 encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf8;

impl fmt::Display for Utf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UTF-8")
    }
}

impl Encoding for Utf8 {
    #[inline]
    fn len(s: &str) -> usize {
        s.len()
    }

    #[inline]
    fn len_char(ch: char) -> usize {
        ch.len_utf8()
    }

    #[inline]
    fn floor_char_boundary(s: &str, index: usize) -> usize {
        s.floor_char_boundary(index)
    }
}

/// UTF-16 encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf16;

impl fmt::Display for Utf16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UTF-16")
    }
}

impl Encoding for Utf16 {
    #[inline]
    fn len(s: &str) -> usize {
        str_indices::utf16::count(s)
    }

    #[inline]
    fn len_char(ch: char) -> usize {
        ch.len_utf16()
    }

    #[inline]
    fn floor_char_boundary(s: &str, index: usize) -> usize {
        str_indices::utf16::to_byte_idx(s, index)
    }

    #[inline]
    fn fits(s: &str, max: usize) -> bool {
        let bytes = s.len();
        // Optimization: The number of UTF-16 code units is always less than the
        // byte length in UTF-8.
        if bytes <= max {
            return true;
        }
        // Optimization: A UTF-16 code unit is at most 3 bytes in UTF-8,
        // so it cannot exceed `3 * max` bytes in UTF-8.
        if bytes > max.saturating_mul(3) {
            return false;
        }
        Self::len(s) <= max
    }
}
