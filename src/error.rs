use std::fmt;

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
    pub(crate) fn invalid_name() -> Error {
        Error {
            kind: Kind::InvalidName,
        }
    }

    pub(crate) fn invalid_value() -> Error {
        Error {
            kind: Kind::InvalidValue,
        }
    }

    pub(crate) fn invalid_path() -> Error {
        Error {
            kind: Kind::InvalidPath,
        }
    }

    pub(crate) fn invalid_domain() -> Error {
        Error {
            kind: Kind::InvalidDomain,
        }
    }

    pub(crate) fn too_long() -> Error {
        Error {
            kind: Kind::TooLong,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::InvalidName => f.write_str("cookie name contains invalid character"),
            Kind::InvalidValue => f.write_str("cookie value contains invalid character"),
            Kind::InvalidPath => f.write_str("cookie path is invalid"),
            Kind::InvalidDomain => f.write_str("cookie domain is invalid"),
            Kind::TooLong => f.write_str("cookie string is too long"),
        }
    }
}

impl std::error::Error for Error {}
