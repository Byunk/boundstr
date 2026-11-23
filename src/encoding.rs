/// A character encoding.
pub trait Encoding {
    /// Returns the length of `s` in this encoding.
    fn len(s: &str) -> usize;

    /// Finds the closest character boundary `x` not exceeding `index`.
    fn floor_char_boundary(s: &str, index: usize) -> usize;
}

/// UTF-8 encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf8;

impl Encoding for Utf8 {
    #[inline]
    fn len(s: &str) -> usize {
        s.len()
    }

    #[inline]
    fn floor_char_boundary(s: &str, index: usize) -> usize {
        s.floor_char_boundary(index)
    }
}

/// UTF-16 encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf16;

impl Encoding for Utf16 {
    #[inline]
    fn len(s: &str) -> usize {
        str_indices::utf16::count(s)
    }

    #[inline]
    fn floor_char_boundary(s: &str, index: usize) -> usize {
        str_indices::utf16::to_byte_idx(s, index)
    }
}
