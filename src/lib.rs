#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! # cookies
//!
//! HTTP cookie parsing and building.
//!
//! # The `Cookie` trait
//!
//! This crate tries something a little different. Instead of representing
//! a cookie with a single `struct`, the *concept* of a cookie is instead
//! exposed as a `trait`.

use std::fmt;
use std::time::Duration;

mod build;
mod error;
mod parse;
mod util;

pub use self::build::Builder;
pub use self::error::Error;
pub use self::parse::parse;
pub use self::util::SameSite;

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

    /// Get the `SameSite`, if set.
    fn same_site(&self) -> Option<SameSite>;
}

mod sealed {
    pub trait Sealed {}
}
