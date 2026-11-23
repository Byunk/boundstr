/*!
A bounded string.
*/

pub mod encoding;

mod bstr;
mod bstring;
mod error;

pub use bstr::BStr;
pub use bstring::BString;
pub use error::FromUtf8Error;
pub use error::FromUtf16Error;
pub use error::LengthExceeded;
