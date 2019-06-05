#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! cookies

use std::fmt;
use std::time::Duration;

mod build;
mod parse;
mod util;

use self::sealed::Sealed;
pub use self::build::Builder;
pub use self::parse::parse;

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

/// Error type when failing to parse or build a `Cookie`.
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    InvalidName,
    InvalidValue,
    InvalidPath,
    InvalidDomain,
    TooLong,
}

// ===== impl Error =====

impl Error {
    fn invalid_name() -> Error {
        Error { kind: Kind::InvalidName }
    }

    fn invalid_value() -> Error {
        Error { kind: Kind::InvalidValue }
    }

    fn invalid_path() -> Error {
        Error { kind: Kind::InvalidPath }
    }

    fn invalid_domain() -> Error {
        Error { kind: Kind::InvalidDomain }
    }

    fn too_long() -> Error {
        Error { kind: Kind::TooLong }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::InvalidName => {
                f.write_str("cookie name contains invalid character")
            },
            Kind::InvalidValue => {
                f.write_str("cookie value contains invalid character")
            },
            Kind::InvalidPath => {
                f.write_str("cookie path is invalid")
            },
            Kind::InvalidDomain => {
                f.write_str("cookie domain is invalid")
            },
            Kind::TooLong => {
                f.write_str("cookie string is too long")
            },
        }
    }
}

impl std::error::Error for Error {}

mod sealed {
    pub trait Sealed {}
}
