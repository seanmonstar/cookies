use std::fmt;
use std::time::Duration;

use super::{Cookie, Error, Sealed};
use crate::util;

/// Configure an HTTP cookie with the builder pattern.
#[derive(Debug)]
pub struct Builder<C> {
    state: Result<C, Error>,
}

struct Pair<N, V>(N, V);

struct WithValue<C, V>(C, V);

struct WithPath<C, P>(C, P);

struct WithDomain<C, D>(C, D);

struct WithMaxAge<C>(C, Duration);

struct WithSecure<C>(C, bool);

struct WithHttpOnly<C>(C, bool);

// ===== impl Builder =====

impl Builder<()> {
    /// Start a new `Builder` with the name and value for a cookie.
    pub fn new(name: impl AsRef<str>, value: impl AsRef<str>) -> Builder<impl Cookie> {
        let state = crate::parse::validate_name(name.as_ref());
        let state = state.and_then(|()| crate::parse::validate_value(value.as_ref()));
        Builder {
            state: state.map(move |()| Pair(name, value)),
        }
    }
}

impl<C: Cookie> Builder<C> {
    /// Wrap an existing `Cookie` in a builder, in order to change attributes.
    pub fn wrap(cookie: C) -> Builder<C> {
        Builder { state: Ok(cookie) }
    }

    /// Set the value of this cookie.
    ///
    /// Useful for overriding a value from a previous cookie.
    pub fn value(self, value: impl AsRef<str>) -> Builder<impl Cookie> {
        self.and_then(move |c| {
            crate::parse::validate_value(value.as_ref())?;
            Ok(util::Delegated(WithValue(c, value)))
        })
    }

    /// Set the `Path` attribute of this cookie.
    pub fn path(self, path: impl AsRef<str>) -> Builder<impl Cookie> {
        self.and_then(move |c| {
            if crate::parse::is_valid_path(path.as_ref()) {
                Ok(util::Delegated(WithPath(c, path)))
            } else {
                // When parsing, invalid paths are just ignored. However,
                // when building a cookie, a user should know when they
                // set something bad.
                Err(Error::invalid_path())
            }
        })
    }

    /// Set the `Domain` attribute of this cookie.
    pub fn domain(self, domain: impl AsRef<str>) -> Builder<impl Cookie> {
        // TODO: validate domain
        self.and_then(move |c| {
            use crate::parse::Domain;
            match crate::parse::validate_domain(domain.as_ref()) {
                Domain::AsIs => Ok(util::Delegated(WithDomain(c, domain))),
                Domain::LeadingDot | Domain::Invalid => Err(Error::invalid_domain()),
            }
        })
    }

    /// Set the `Max-Age` attribute of this cookie.
    pub fn max_age(self, max_age: Duration) -> Builder<impl Cookie> {
        self.and_then(move |c| Ok(util::Delegated(WithMaxAge(c, max_age))))
    }

    /// Enable or disable the `Secure` attribute of this cookie.
    pub fn secure(self, secure: bool) -> Builder<impl Cookie> {
        self.and_then(move |c| Ok(util::Delegated(WithSecure(c, secure))))
    }

    /// Enable or disable the `HttpOnly` attribute of this cookie.
    pub fn http_only(self, http_only: bool) -> Builder<impl Cookie> {
        self.and_then(move |c| Ok(util::Delegated(WithHttpOnly(c, http_only))))
    }

    /// Consumes the builder trying to return the constructed `Cookie`.
    ///
    /// # Error
    ///
    /// Returns an error if any of the builder steps were passed an invalid
    /// value.
    pub fn build(self) -> Result<C, Error> {
        self.state
    }

    // private

    fn and_then<F, R>(self, func: F) -> Builder<impl Cookie>
    where
        F: FnOnce(C) -> Result<R, Error>,
        R: Cookie,
    {
        Builder {
            state: self.state.and_then(func),
        }
    }
}

// ===== impl Pair =====

impl<N: AsRef<str>, V: AsRef<str>> Cookie for Pair<N, V> {
    fn name(&self) -> &str {
        self.0.as_ref()
    }

    fn value(&self) -> &str {
        self.1.as_ref()
    }

    fn domain(&self) -> Option<&str> {
        None
    }

    fn path(&self) -> Option<&str> {
        None
    }

    fn max_age(&self) -> Option<Duration> {
        None
    }

    fn http_only(&self) -> bool {
        false
    }

    fn secure(&self) -> bool {
        false
    }

    fn same_site_strict(&self) -> bool {
        false
    }

    fn same_site_lax(&self) -> bool {
        false
    }
}

impl<N, V> Sealed for Pair<N, V> {}

impl<N: AsRef<str>, V: AsRef<str>> fmt::Debug for Pair<N, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        util::debug(self, f)
    }
}

impl<N: AsRef<str>, V: AsRef<str>> fmt::Display for Pair<N, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        util::display(self, f)
    }
}

// ===== impl WithValue =====

impl<C: Cookie, V: AsRef<str>> util::Delegate for WithValue<C, V> {
    type Cookie = C;
    fn cookie(&self) -> &Self::Cookie {
        &self.0
    }

    fn value(&self) -> &str {
        self.1.as_ref()
    }
}

// ===== impl WithPath =====

impl<C: Cookie, P: AsRef<str>> util::Delegate for WithPath<C, P> {
    type Cookie = C;
    fn cookie(&self) -> &Self::Cookie {
        &self.0
    }

    fn path(&self) -> Option<&str> {
        Some(self.1.as_ref())
    }
}

// ===== impl WithDomain =====

impl<C: Cookie, D: AsRef<str>> util::Delegate for WithDomain<C, D> {
    type Cookie = C;
    fn cookie(&self) -> &Self::Cookie {
        &self.0
    }

    fn domain(&self) -> Option<&str> {
        Some(self.1.as_ref())
    }
}

// ===== impl WithMaxAge =====

impl<C: Cookie> util::Delegate for WithMaxAge<C> {
    type Cookie = C;
    fn cookie(&self) -> &Self::Cookie {
        &self.0
    }

    fn max_age(&self) -> Option<Duration> {
        Some(self.1)
    }
}

// ===== impl WithSecure =====

impl<C: Cookie> util::Delegate for WithSecure<C> {
    type Cookie = C;
    fn cookie(&self) -> &Self::Cookie {
        &self.0
    }

    fn secure(&self) -> bool {
        self.1
    }
}

// ===== impl WithHttpOnly =====

impl<C: Cookie> util::Delegate for WithHttpOnly<C> {
    type Cookie = C;
    fn cookie(&self) -> &Self::Cookie {
        &self.0
    }

    fn http_only(&self) -> bool {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair() {
        // builder with no other attributes
        let c = Builder::new("foo", "bar").build().unwrap();
        assert_eq!(c.name(), "foo");
        assert_eq!(c.value(), "bar");
        assert_eq!(c.path(), None);
        assert_eq!(c.domain(), None);
        assert_eq!(c.max_age(), None);
        assert!(!c.http_only());
        assert!(!c.secure());
        assert!(!c.same_site_strict());
        assert!(!c.same_site_lax());
    }

    #[test]
    fn pair_validates() {
        Builder::new("", "bar").build().expect_err("empty name");
        Builder::new("foo=", "bar")
            .build()
            .expect_err("invalid name");
        Builder::new("foo", "bar\n")
            .build()
            .expect_err("invalid value");
    }

    #[test]
    fn with_value() {
        // can change the value
        let c = Builder::new("foo", "bar").value("wat").build().unwrap();

        assert_eq!(c.value(), "wat");
    }

    #[test]
    fn with_path() {
        let c = Builder::new("foo", "bar").path("/hallo").build().unwrap();

        assert_eq!(c.path(), Some("/hallo"));

        let c2 = Builder::wrap(c).path("/bye").build().unwrap();

        assert_eq!(c2.path(), Some("/bye"));

        Builder::new("foo", "bar")
            .path("bad-path")
            .build()
            .expect_err("path without leading slash");

        Builder::new("foo", "bar")
            .path("/hello\nwat")
            .build()
            .expect_err("path with CTL");
    }

    #[test]
    fn with_domain() {
        let c = Builder::new("foo", "bar")
            .domain("hyper.rs")
            .build()
            .unwrap();

        assert_eq!(c.domain(), Some("hyper.rs"));

        Builder::new("foo", "bar")
            .domain("hyper\nrs")
            .build()
            .expect_err("domain with CTL");

        Builder::new("foo", "bar")
            .domain(".hyper.rs")
            .build()
            .expect_err("domain with leading dot");
    }

    #[test]
    fn with_max_age() {
        let c = Builder::new("foo", "bar")
            .max_age(Duration::from_secs(10))
            .build()
            .unwrap();

        assert_eq!(c.max_age(), Some(Duration::from_secs(10)));
    }
}
