use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, RangeBounds};
use std::str::FromStr;

use crate::bstr::BStr;
use crate::encoding::{Encoding, Utf8};
use crate::error::{FromUtf8Error, FromUtf16Error, LengthExceeded};

/// A bounded [`String`] whose length is at most `MAX` in encoding `E`.
pub struct BString<const MAX: usize, E: Encoding = Utf8> {
    s: String,
    /// The remaining budget for the string length, in encoding `E`.
    budget: usize,
    phantom: PhantomData<E>,
}

impl<const MAX: usize, E: Encoding> BString<MAX, E> {
    /// The maximum length, in encoding `E`.
    pub const MAX: usize = MAX;

    /// Creates an empty [`BString`].
    pub const fn new() -> Self {
        Self {
            s: String::new(),
            budget: MAX,
            phantom: PhantomData,
        }
    }

    /// Converts a vector of bytes to a [`BString`].
    ///
    /// # Errors
    ///
    /// Returns [`FromUtf8Error`] if `vec` is not valid UTF-8 or the string
    /// exceeds `MAX` in encoding `E`.
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error<E>> {
        let s = String::from_utf8(vec)?;
        Ok(Self::from_string(s)?)
    }

    /// Decodes a UTF-16–encoded slice into a [`BString`].
    ///
    /// # Errors
    ///
    /// Returns [`FromUtf16Error`] if `v` is not valid UTF-16 or the string
    /// exceeds `MAX` in encoding `E`.
    pub fn from_utf16(v: &[u16]) -> Result<Self, FromUtf16Error<E>> {
        let s = String::from_utf16(v)?;
        Ok(Self::from_string(s)?)
    }

    /// Converts a [`String`] to a [`BString`].
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `s` exceeds `MAX` in encoding `E`.
    pub fn from_string(s: String) -> Result<Self, LengthExceeded<E>> {
        let len = E::len(&s);
        if len > MAX {
            return Err(LengthExceeded::new(len, MAX));
        }
        Ok(Self {
            s,
            budget: MAX - len,
            phantom: PhantomData,
        })
    }

    /// Converts a [`String`] to a [`BString`], truncating it to at most `MAX`
    /// in encoding `E`.
    ///
    /// This method ensures that the [`BString`] is still valid UTF-8 after
    /// truncation. If the given `s` cannot be truncated to exactly `MAX`, the
    /// string is truncated at the last valid character boundary in encoding `E`
    /// that does not exceed `MAX`. So, the resulting [`BString`] may be shorter
    /// than `MAX`.
    ///
    /// This method has no effect on the allocated capacity of the string.
    pub fn from_string_truncated(mut s: String) -> Self {
        s.truncate(E::floor_char_boundary(&s, MAX));
        let len = E::len(&s);
        Self {
            s,
            budget: MAX - len,
            phantom: PhantomData,
        }
    }

    /// Appends `ch` to the end of this [`BString`].
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `ch` would make the string exceed `MAX` in
    /// encoding `E`.
    pub fn try_push(&mut self, ch: char) -> Result<(), LengthExceeded<E>> {
        self.try_insert(self.s.len(), ch)
    }

    /// Appends `s` to the end of this [`BString`].
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `s` would make the string exceed `MAX` in
    /// encoding `E`.
    pub fn try_push_str(&mut self, s: &str) -> Result<(), LengthExceeded<E>> {
        self.try_insert_str(self.s.len(), s)
    }

    /// Appends `s` to the end of this [`BString`], truncating `s` so that the
    /// string is at most `MAX` in encoding `E`.
    ///
    /// This method ensures that the [`BString`] is still valid UTF-8 after
    /// truncation. If the given `s` cannot be truncated to reach exactly `MAX`,
    /// `s` is truncated at the last valid character boundary in encoding `E`
    /// that does not make the string exceed `MAX`. So, the resulting
    /// [`BString`] may be shorter than `MAX`.
    ///
    /// The returned slice is empty if all of `s` fit.
    pub fn push_str_truncated<'a>(&mut self, s: &'a str) -> &'a str {
        self.insert_str_truncated(self.s.len(), s)
    }

    /// Inserts `ch` at byte position `idx`.
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `ch` would make the string exceed `MAX` in
    /// encoding `E`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the string's length in bytes, or if it
    /// does not lie on a char boundary.
    pub fn try_insert(&mut self, idx: usize, ch: char) -> Result<(), LengthExceeded<E>> {
        self.try_insert_str(idx, ch.encode_utf8(&mut [0; 4]))
    }

    /// Inserts `s` at byte position `idx`.
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `s` would make the string exceed `MAX` in
    /// encoding `E`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the string's length in bytes, or if it
    /// does not lie on a char boundary.
    pub fn try_insert_str(&mut self, idx: usize, s: &str) -> Result<(), LengthExceeded<E>> {
        let added = E::len(s);
        if added > self.budget {
            return Err(LengthExceeded::new(MAX - self.budget + added, MAX));
        }
        self.s.insert_str(idx, s);
        self.budget -= added;
        Ok(())
    }

    /// Inserts `s` at byte position `idx`, truncating `s` so that the string is
    /// at most `MAX` in encoding `E`.
    ///
    /// This method ensures that the [`BString`] is still valid UTF-8 after
    /// truncation. If the given `s` cannot be truncated to reach exactly `MAX`,
    /// `s` is truncated at the last valid character boundary in encoding `E`
    /// that does not make the string exceed `MAX`. So, the resulting
    /// [`BString`] may be shorter than `MAX`.
    ///
    /// The returned slice is empty if all of `s` fit.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the string's length in bytes, or if it
    /// does not lie on a char boundary.
    pub fn insert_str_truncated<'a>(&mut self, idx: usize, s: &'a str) -> &'a str {
        let (head, rest) = s.split_at(E::floor_char_boundary(s, self.budget));
        self.s.insert_str(idx, head);
        self.budget -= E::len(head);
        rest
    }

    /// Replaces the given byte range with `with`.
    ///
    /// # Errors
    ///
    /// Returns [`LengthExceeded`] if `with` would make the string exceed `MAX`
    /// in encoding `E`.
    ///
    /// # Panics
    ///
    /// Panics if the start or end of the range is out of bounds or does not
    /// lie on a char boundary.
    pub fn try_replace_range<R: RangeBounds<usize>>(
        &mut self,
        range: R,
        with: &str,
    ) -> Result<(), LengthExceeded<E>> {
        let range = (range.start_bound().cloned(), range.end_bound().cloned());
        let removed = E::len(&self.s[range]);
        let added = E::len(with);
        let room = self.budget + removed;
        if added > room {
            return Err(LengthExceeded::new(MAX - room + added, MAX));
        }
        self.s.replace_range(range, with);
        self.budget = room - added;
        Ok(())
    }

    /// Replaces the given byte range with `with`, truncating `with` so that the
    /// string is at most `MAX` in encoding `E`.
    ///
    /// The `range` is always removed in full. So, the room for `with` is what
    /// the range frees plus what was already remaining.
    ///
    /// This method ensures that the [`BString`] is still valid UTF-8 after
    /// truncation. If the given `with` cannot be truncated to reach exactly
    /// `MAX`, `with` is truncated at the last valid character boundary in
    /// encoding `E` that does not make the string exceed `MAX`. So, the
    /// resulting [`BString`] may be shorter than `MAX`.
    ///
    /// The returned slice is empty if all of `with` fit.
    ///
    /// # Panics
    ///
    /// Panics if the start or end of the range is out of bounds or does not
    /// lie on a char boundary.
    pub fn replace_range_truncated<'a, R>(&mut self, range: R, with: &'a str) -> &'a str
    where
        R: RangeBounds<usize>,
    {
        let range = (range.start_bound().cloned(), range.end_bound().cloned());
        let removed = E::len(&self.s[range]);
        let room = self.budget + removed;
        let (head, rest) = with.split_at(E::floor_char_boundary(with, room));
        self.s.replace_range(range, head);
        self.budget = room - E::len(head);
        rest
    }

    /// Truncates this [`BString`], removing all contents.
    ///
    /// This method has no effect on the allocated capacity of the string.
    pub fn clear(&mut self) {
        self.s.clear();
        self.budget = MAX;
    }

    /// Removes the last character from this [`BString`] and returns it.
    ///
    /// Returns [`None`] if this [`BString`] is empty.
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.s.pop()?;
        self.budget += E::len(ch.encode_utf8(&mut [0; 4]));
        Some(ch)
    }

    /// Removes the character at byte position `idx` from this [`BString`] and
    /// returns it.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the string's length in bytes,
    /// or if it does not lie on a char boundary.
    pub fn remove(&mut self, idx: usize) -> char {
        let ch = self.s.remove(idx);
        self.budget += E::len(ch.encode_utf8(&mut [0; 4]));
        ch
    }

    /// Shortens this [`BString`] to `new_len` bytes.
    ///
    /// If `new_len` is greater than or equal to the string's current length in
    /// bytes, this has no effect.
    ///
    /// This method has no effect on the allocated capacity of the string.
    ///
    /// # Panics
    ///
    /// Panics if `new_len` does not lie on a char boundary.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.s.len() {
            let removed = E::len(&self.s[new_len..]);
            self.s.truncate(new_len);
            self.budget += removed;
        }
    }

    /// Splits this [`BString`] into two at byte position `at`.
    ///
    /// Returns a newly allocated [`BString`]. `self` contains bytes
    /// `[0, at)`, and the returned [`BString`] contains bytes `[at, len)`.
    ///
    /// This method has no effect on the allocated capacity of `self`.
    ///
    /// # Panics
    ///
    /// Panics if `at` is larger than the string's length in bytes, or if it
    /// does not lie on a char boundary.
    #[must_use = "use `.truncate()` if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> Self {
        let tail = self.s.split_off(at);
        let moved = E::len(&tail);
        self.budget += moved;
        Self {
            s: tail,
            budget: MAX - moved,
            phantom: PhantomData,
        }
    }

    /// Retains only the characters for which `f` returns `true`.
    ///
    /// This method operates in place, visiting each character exactly once in
    /// the original order, and preserves the order of the retained characters.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(char) -> bool,
    {
        self.s.retain(f);
        self.budget = MAX - E::len(&self.s);
    }

    /// Returns a [`BStr`] slice of the entire string.
    pub const fn as_bstr(&self) -> &BStr<MAX, E> {
        BStr::from_str_unchecked(self.s.as_str())
    }

    /// Returns a [`str`] slice of the entire string.
    pub const fn as_str(&self) -> &str {
        self.s.as_str()
    }

    /// Converts this [`BString`] into a [`String`].
    pub fn into_string(self) -> String {
        self.s
    }
}

impl<const MAX: usize, E: Encoding> fmt::Debug for BString<MAX, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.s, f)
    }
}

impl<const MAX: usize, E: Encoding> fmt::Display for BString<MAX, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.s, f)
    }
}

impl<const MAX: usize, E: Encoding> Default for BString<MAX, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX: usize, E: Encoding> Clone for BString<MAX, E> {
    fn clone(&self) -> Self {
        Self {
            s: self.s.clone(),
            budget: self.budget,
            phantom: PhantomData,
        }
    }
}

impl<const MAX: usize, E: Encoding> PartialEq for BString<MAX, E> {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}

impl<const MAX: usize, E: Encoding> Eq for BString<MAX, E> {}

impl<const MAX: usize, E: Encoding> PartialOrd for BString<MAX, E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const MAX: usize, E: Encoding> Ord for BString<MAX, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.s.cmp(&other.s)
    }
}

impl<const MAX: usize, E: Encoding> Hash for BString<MAX, E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.s.hash(state);
    }
}

impl<const MAX: usize, E: Encoding> Deref for BString<MAX, E> {
    type Target = BStr<MAX, E>;

    fn deref(&self) -> &BStr<MAX, E> {
        self.as_bstr()
    }
}

impl<const MAX: usize, E: Encoding> AsRef<BStr<MAX, E>> for BString<MAX, E> {
    fn as_ref(&self) -> &BStr<MAX, E> {
        self.as_bstr()
    }
}

impl<const MAX: usize, E: Encoding> AsRef<str> for BString<MAX, E> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const MAX: usize, E: Encoding> Borrow<BStr<MAX, E>> for BString<MAX, E> {
    fn borrow(&self) -> &BStr<MAX, E> {
        self.as_bstr()
    }
}

impl<const MAX: usize, E: Encoding> Borrow<str> for BString<MAX, E> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const MAX: usize, E: Encoding> ToOwned for BStr<MAX, E> {
    type Owned = BString<MAX, E>;

    fn to_owned(&self) -> BString<MAX, E> {
        BString {
            s: String::from(&**self),
            budget: MAX - E::len(self),
            phantom: PhantomData,
        }
    }
}

impl<const MAX: usize, E: Encoding> From<&BStr<MAX, E>> for BString<MAX, E> {
    fn from(s: &BStr<MAX, E>) -> Self {
        s.to_owned()
    }
}

impl<const MAX: usize, E: Encoding> From<BString<MAX, E>> for String {
    fn from(s: BString<MAX, E>) -> Self {
        s.s
    }
}

impl<const MAX: usize, E: Encoding> TryFrom<String> for BString<MAX, E> {
    type Error = LengthExceeded<E>;

    fn try_from(s: String) -> Result<Self, LengthExceeded<E>> {
        Self::from_string(s)
    }
}

impl<const MAX: usize, E: Encoding> TryFrom<&str> for BString<MAX, E> {
    type Error = LengthExceeded<E>;

    fn try_from(s: &str) -> Result<Self, LengthExceeded<E>> {
        Self::from_string(s.to_owned())
    }
}

impl<const MAX: usize, E: Encoding> FromStr for BString<MAX, E> {
    type Err = LengthExceeded<E>;

    fn from_str(s: &str) -> Result<Self, LengthExceeded<E>> {
        Self::from_string(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::encoding::Utf16;

    fn bstring<const MAX: usize, E: Encoding>(s: &str) -> BString<MAX, E> {
        BString::from_string(s.to_owned()).unwrap()
    }

    /// The content and the remaining budget of `s`.
    fn state<const MAX: usize, E: Encoding>(s: &BString<MAX, E>) -> (&str, usize) {
        (s.as_str(), s.budget)
    }

    #[test]
    fn from_string_truncated() {
        fn truncated<const MAX: usize, E: Encoding>(s: &str) -> BString<MAX, E> {
            BString::from_string_truncated(s.to_owned())
        }

        assert_eq!(state(&truncated::<10, Utf8>("hello")), ("hello", 5));
        assert_eq!(state(&truncated::<3, Utf8>("hello")), ("hel", 0));
        assert_eq!(state(&truncated::<0, Utf8>("hello")), ("", 0));
        // 'é' is 2 bytes in UTF-8 but 1 code unit in UTF-16.
        assert_eq!(state(&truncated::<2, Utf8>("éé")), ("é", 0));
        assert_eq!(state(&truncated::<1, Utf16>("éé")), ("é", 0));
        assert_eq!(state(&truncated::<3, Utf16>("a😀b")), ("a😀", 0));
        // `MAX` falls inside the 2-byte 'é'.
        assert_eq!(state(&truncated::<2, Utf8>("aé")), ("a", 1));
        // `MAX` falls between the two surrogates of '😀'.
        assert_eq!(state(&truncated::<2, Utf16>("a😀b")), ("a", 1));
    }

    #[test]
    fn insert_str_truncated() {
        // All of `s` fits.
        let mut s = bstring::<5, Utf8>("ad");
        assert_eq!(s.insert_str_truncated(1, "bc"), "");
        assert_eq!(state(&s), ("abcd", 1));

        // Only a prefix of `s` fits.
        let mut s = bstring::<5, Utf8>("ad");
        assert_eq!(s.insert_str_truncated(1, "bcxyz"), "yz");
        assert_eq!(state(&s), ("abcxd", 0));

        // Nothing fits.
        let mut s = bstring::<2, Utf8>("ad");
        assert_eq!(s.insert_str_truncated(1, "x"), "x");
        assert_eq!(state(&s), ("ad", 0));

        // The budget runs out inside the 2-byte 'é'.
        let mut s = bstring::<4, Utf8>("ab");
        assert_eq!(s.insert_str_truncated(1, "xé"), "é");
        assert_eq!(state(&s), ("axb", 1));

        // '😀' is 2 code units in UTF-16.
        let mut s = bstring::<4, Utf16>("ab");
        assert_eq!(s.insert_str_truncated(1, "😀c"), "c");
        assert_eq!(state(&s), ("a😀b", 0));

        // The budget runs out between the two surrogates of '😀'.
        let mut s = bstring::<3, Utf16>("ab");
        assert_eq!(s.insert_str_truncated(2, "😀"), "😀");
        assert_eq!(state(&s), ("ab", 1));
    }

    #[test]
    fn replace_range_truncated() {
        // All of `with` fits.
        let mut s = bstring::<5, Utf8>("hello");
        assert_eq!(s.replace_range_truncated(1..4, "EY"), "");
        assert_eq!(state(&s), ("hEYo", 1));

        // Only a prefix of `with` fits in what the range frees.
        let mut s = bstring::<5, Utf8>("hello");
        assert_eq!(s.replace_range_truncated(1..3, "WXYZ"), "YZ");
        assert_eq!(state(&s), ("hWXlo", 0));

        // An unbounded range replaces the whole string.
        let mut s = bstring::<3, Utf8>("abc");
        assert_eq!(s.replace_range_truncated(.., "wxyz"), "z");
        assert_eq!(state(&s), ("wxy", 0));

        // The range is removed in full even if nothing of `with` fits: the
        // room runs out inside the 2-byte 'é'.
        let mut s = bstring::<3, Utf8>("abc");
        assert_eq!(s.replace_range_truncated(1..2, "é"), "é");
        assert_eq!(state(&s), ("ac", 1));

        // '😀' is 4 bytes in UTF-8 but 2 code units in UTF-16.
        let mut s = bstring::<3, Utf16>("a😀");
        assert_eq!(s.replace_range_truncated(1..5, "éé!"), "!");
        assert_eq!(state(&s), ("aéé", 0));

        // The room runs out between the two surrogates of '😀'.
        let mut s = bstring::<3, Utf16>("abc");
        assert_eq!(s.replace_range_truncated(1..2, "😀"), "😀");
        assert_eq!(state(&s), ("ac", 1));
    }
}
