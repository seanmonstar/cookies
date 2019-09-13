#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! cookies

use std::fmt;
use std::time::Duration;

mod build;
mod error;
mod parse;
mod util;

pub use self::build::Builder;
pub use self::error::Error;
pub use self::parse::parse;
use self::sealed::Sealed;

/// Cookies in this crate implement this trait.
pub trait Cookie: fmt::Debug + fmt::Display + Sealed {
    /// Get the name of this cookie.
    fn name(&self) -> &str;

    /// Get the value of this cookie.
    fn value(&self) -> &str;

    /// Get the `Domain`, if set.
    fn domain(&self) -> Option<&str>;

    /// Get the `Path`, if set.
    fn path(&self) -> Option<&str>;

    /// Get the `Max-Age`, if set.
    fn max_age(&self) -> Option<Duration>;

    /// Get if the `HttpOnly` attribute was on this cookie.
    fn http_only(&self) -> bool;

    /// Get if the `Secure` attribute was on this cookie.
    fn secure(&self) -> bool;

    /// Get if the `SameSite=Strict` attribute was on this cookie.
    fn same_site_strict(&self) -> bool;

    /// Get if the `SameSite=Lax` attribute was on this cookie.
    fn same_site_lax(&self) -> bool;
}

mod sealed {
    pub trait Sealed {}
}
