use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;

use crate::encoding::{Encoding, Utf8};
use crate::error::LengthExceeded;

/// A bounded [`str`] whose length is at most `MAX` in encoding `E`.
#[repr(transparent)]
pub struct BStr<const MAX: usize, E = Utf8> {
    phantom: PhantomData<E>,
    s: str,
}

impl<const MAX: usize, E: Encoding> BStr<MAX, E> {
    /// The maximum length, in encoding `E`.
    pub const MAX: usize = MAX;

    /// Converts a `&str` to a [`BStr`].
    ///
    /// Takes `O(n)` time complexity.
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `s` exceeds `MAX` in encoding `E`.
    pub fn from_str(s: &str) -> Result<&Self, LengthExceeded<E>> {
        let len = E::len(s);
        if len > MAX {
            return Err(LengthExceeded::new(len, MAX));
        }
        Ok(Self::from_str_unchecked(s))
    }

    /// Converts a `&str` to a [`BStr`], truncating it to at most `MAX` in
    /// encoding `E`.
    ///
    /// This method ensures that the [`BStr`] is still valid UTF-8 after
    /// truncation. If the given `s` cannot be truncated to exactly `MAX`, the
    /// string is truncated at the last valid character boundary in encoding `E`
    /// that does not exceed `MAX`. So, the resulting [`BStr`] may be shorter
    /// than `MAX`.
    pub fn from_str_truncated(s: &str) -> &Self {
        let end = E::floor_char_boundary(s, MAX);
        Self::from_str_unchecked(&s[..end])
    }

    /// Converts a `&str` to a [`BStr`], without checking its length.
    ///
    /// It's for internal use only, and the caller must ensure that `s` does not
    /// exceed `MAX` in encoding `E`.
    pub(crate) const fn from_str_unchecked(s: &str) -> &Self {
        // SAFETY: `BStr`and `str` have the same memory layout.
        unsafe { &*(s as *const str as *const Self) }
    }
}

impl<const MAX: usize, E> fmt::Debug for BStr<MAX, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.s, f)
    }
}

impl<const MAX: usize, E> fmt::Display for BStr<MAX, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.s, f)
    }
}

impl<const MAX: usize, E: Encoding> Default for &BStr<MAX, E> {
    fn default() -> Self {
        BStr::from_str_unchecked("")
    }
}

impl<const MAX: usize, E> PartialEq for BStr<MAX, E> {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}

impl<const MAX: usize, E> Eq for BStr<MAX, E> {}

impl<const MAX: usize, E> PartialOrd for BStr<MAX, E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const MAX: usize, E> Ord for BStr<MAX, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.s.cmp(&other.s)
    }
}

impl<const MAX: usize, E> Hash for BStr<MAX, E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.s.hash(state);
    }
}

impl<const MAX: usize, E> Deref for BStr<MAX, E> {
    type Target = str;

    fn deref(&self) -> &str {
        &self.s
    }
}

impl<const MAX: usize, E> AsRef<str> for BStr<MAX, E> {
    fn as_ref(&self) -> &str {
        &self.s
    }
}

impl<const MAX: usize, E> Borrow<str> for BStr<MAX, E> {
    fn borrow(&self) -> &str {
        &self.s
    }
}

impl<'a, const MAX: usize, E: Encoding> TryFrom<&'a str> for &'a BStr<MAX, E> {
    type Error = LengthExceeded<E>;

    fn try_from(s: &'a str) -> Result<Self, LengthExceeded<E>> {
        BStr::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::Utf16;

    use super::*;

    fn truncated<const MAX: usize, E: Encoding + 'static>(s: &str) -> &str {
        BStr::<MAX, E>::from_str_truncated(s)
    }

    #[test]
    fn from_str_truncated() {
        assert_eq!(truncated::<3, Utf8>("hello"), "hel");
        assert_eq!(truncated::<0, Utf8>("hello"), "");
        // 'é' is 2 bytes in UTF-8 but 1 code unit in UTF-16.
        assert_eq!(truncated::<2, Utf8>("éé"), "é");
        assert_eq!(truncated::<1, Utf16>("éé"), "é");
        assert_eq!(truncated::<3, Utf16>("a😀b"), "a😀");
    }

    #[test]
    fn from_str_truncated_never_splits_a_char() {
        // `MAX` falls inside the 2-byte 'é'.
        assert_eq!(truncated::<2, Utf8>("aé"), "a");
        // `MAX` falls inside the 4-byte '😀'.
        assert_eq!(truncated::<4, Utf8>("a😀"), "a");
        // `MAX` falls between the two surrogates of '😀'.
        assert_eq!(truncated::<2, Utf16>("a😀b"), "a");
    }
}
