/*!
A bounded string.
*/

pub mod encoding;

mod bstr;
mod error;

pub use bstr::BStr;
pub use error::LengthExceeded;
