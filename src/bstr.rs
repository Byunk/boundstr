use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, Index};
use std::slice::SliceIndex;

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

    /// Returns a subslice of this [`BStr`], or [`None`] if indexing would
    /// panic.
    ///
    /// See [`str::get`] for details.
    #[inline]
    pub fn get<I: SliceIndex<str, Output = str>>(&self, i: I) -> Option<&Self> {
        self.s.get(i).map(Self::from_str_unchecked)
    }

    /// Divides this [`BStr`] into two at byte position `mid`.
    ///
    /// See [`str::split_at`] for details.
    ///
    /// # Panics
    ///
    /// Panics if `mid` is larger than the string's length in bytes, or if it
    /// does not lie on a char boundary.
    #[inline]
    #[must_use]
    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        let (first, last) = self.s.split_at(mid);
        (
            Self::from_str_unchecked(first),
            Self::from_str_unchecked(last),
        )
    }

    /// Divides this [`BStr`] into two at byte position `mid`, or returns
    /// [`None`] if splitting would panic.
    ///
    /// See [`str::split_at_checked`] for details.
    #[inline]
    #[must_use]
    pub fn split_at_checked(&self, mid: usize) -> Option<(&Self, &Self)> {
        let (first, last) = self.s.split_at_checked(mid)?;
        Some((
            Self::from_str_unchecked(first),
            Self::from_str_unchecked(last),
        ))
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

impl<const MAX: usize, E: Encoding, I> Index<I> for BStr<MAX, E>
where
    I: SliceIndex<str, Output = str>,
{
    type Output = Self;

    #[inline]
    fn index(&self, index: I) -> &Self {
        Self::from_str_unchecked(&self.s[index])
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

#[cfg(feature = "serde")]
mod serde_impl {
    use std::fmt;
    use std::marker::PhantomData;

    use serde_core::de::{self, Deserialize, Deserializer, Unexpected, Visitor};
    use serde_core::ser::{Serialize, Serializer};

    use super::BStr;
    use crate::encoding::Encoding;

    impl<const MAX: usize, E> Serialize for BStr<MAX, E> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self)
        }
    }

    struct BStrVisitor<const MAX: usize, E>(PhantomData<E>);

    impl<'de, const MAX: usize, E: Encoding + 'de> Visitor<'de> for BStrVisitor<MAX, E> {
        type Value = &'de BStr<MAX, E>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "a borrowed string of at most {MAX} in {}", E::default())
        }

        fn visit_borrowed_str<Err: de::Error>(self, v: &'de str) -> Result<Self::Value, Err> {
            BStr::from_str(v).map_err(Err::custom)
        }

        fn visit_borrowed_bytes<Err: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, Err> {
            match str::from_utf8(v) {
                Ok(s) => self.visit_borrowed_str(s),
                Err(_) => Err(Err::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }
    }

    impl<'de: 'a, 'a, const MAX: usize, E: Encoding + 'de> Deserialize<'de> for &'a BStr<MAX, E> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_str(BStrVisitor::<MAX, E>(PhantomData))
        }
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

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        use serde_core::de::Deserialize;
        use serde_core::de::value::{BorrowedStrDeserializer, Error, StrDeserializer};

        fn borrowed<const MAX: usize>(s: &str) -> Result<&BStr<MAX, Utf16>, Error> {
            Deserialize::deserialize(BorrowedStrDeserializer::new(s))
        }

        assert_eq!(borrowed::<3>("a😀").unwrap(), "a😀");
        assert!(borrowed::<2>("a😀").is_err());

        // A string that does not outlive the deserializer cannot be borrowed.
        let transient: Result<&BStr<3, Utf16>, Error> =
            Deserialize::deserialize(StrDeserializer::new("a😀"));
        assert!(transient.is_err());
    }
}
