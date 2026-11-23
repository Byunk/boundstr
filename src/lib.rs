/*!
A bounded string that can never grow past a fixed maximum length.

# Example

```
use boundstr::BString;

let mut quote: BString<16> = "Love is".parse()?;
assert!(quote.try_push_str(" a continual interrogation.").is_err());
assert_eq!(quote.as_str(), "Love is");

// Keep what fits, and get back what didn't.
let rest = quote.push_str_truncated(" a continual interrogation.");
assert_eq!(quote.as_str(), "Love is a contin");
assert_eq!(rest, "ual interrogation.");

# Ok::<(), Box<dyn std::error::Error>>(())
```

Truncation never splits a character. It cuts at the last character boundary
that still fits, so the result may be shorter than the maximum.

```
use boundstr::{BStr, encoding::Utf16};

// '🐈' is 4 bytes in UTF-8, but only 3 are left.
let s = BStr::<8>::from_str_truncated("Box: 🐈");
assert_eq!(s.len(), 5);
assert_eq!(s, "Box: ");

// '🐈' is 2 code units in UTF-16, but only 1 is left.
let s = BStr::<6, Utf16>::from_str_truncated("Box: 🐈");
assert_eq!(s, "Box: ");

// The cat is either in the box or not. Never half of it.
let s = BStr::<7, Utf16>::from_str_truncated("Box: 🐈");
assert_eq!(s, "Box: 🐈");
```

# Encodings

[`BString`] and [`BStr`] store strings as UTF-8 internally, regardless of the
[`Encoding`](encoding::Encoding) used to measure length.

# Crate features

* **serde** - Implements `Serialize` and `Deserialize` for [`BString`] and
  [`BStr`].
*/

pub mod encoding;

mod bstr;
mod bstring;
mod error;
mod impls;

pub use bstr::BStr;
pub use bstring::BString;
pub use error::FromUtf8Error;
pub use error::FromUtf16Error;
pub use error::LengthExceeded;
