use std::fmt;
use std::time::Duration;

use super::{Cookie, Sealed};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SameSite {
    Lax,
    Strict,
}

pub(crate) struct Delegated<D>(pub(crate) D);

/// A delegate/proxy of the `Cookie` trait.
///
/// Allows a simple wrapper to just override a certain method,
/// and otherwise delegate to an inner `Cookie`.
pub(crate) trait Delegate {
    type Cookie: Cookie;

    fn cookie(&self) -> &Self::Cookie;

    fn name(&self) -> &str {
        self.cookie().name()
    }

    fn value(&self) -> &str {
        self.cookie().value()
    }

    fn domain(&self) -> Option<&str> {
        self.cookie().domain()
    }

    fn path(&self) -> Option<&str> {
        self.cookie().path()
    }

    fn max_age(&self) -> Option<Duration> {
        self.cookie().max_age()
    }

    fn http_only(&self) -> bool {
        self.cookie().http_only()
    }

    fn secure(&self) -> bool {
        self.cookie().secure()
    }

    fn same_site_strict(&self) -> bool {
        self.cookie().same_site_strict()
    }

    fn same_site_lax(&self) -> bool {
        self.cookie().same_site_lax()
    }
}

impl<D: Delegate> Cookie for Delegated<D> {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn value(&self) -> &str {
        self.0.value()
    }

    fn domain(&self) -> Option<&str> {
        self.0.domain()
    }

    fn path(&self) -> Option<&str> {
        self.0.path()
    }

    fn max_age(&self) -> Option<Duration> {
        self.0.max_age()
    }

    fn http_only(&self) -> bool {
        self.0.http_only()
    }

    fn secure(&self) -> bool {
        self.0.secure()
    }

    fn same_site_strict(&self) -> bool {
        self.0.same_site_strict()
    }

    fn same_site_lax(&self) -> bool {
        self.0.same_site_lax()
    }
}

impl<D: Delegate> Sealed for Delegated<D> {}

impl<D: Delegate> fmt::Debug for Delegated<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug(self, f)
    }
}

impl<D: Delegate> fmt::Display for Delegated<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display(self, f)
    }
}

pub(crate) fn debug(cookie: &dyn Cookie, f: &mut fmt::Formatter) -> fmt::Result {
    let mut builder = f.debug_struct("Cookie");
    builder
        .field("name", &cookie.name())
        .field("value", &cookie.value());

    if let Some(ref path) = cookie.path() {
        builder.field("path", path);
    }

    if let Some(ref domain) = cookie.domain() {
        builder.field("domain", domain);
    }

    if let Some(ref ma) = cookie.max_age() {
        builder.field("max_age", ma);
    }

    if cookie.http_only() {
        builder.field("http_only", &true);
    }

    if cookie.secure() {
        builder.field("secure", &true);
    }

    if cookie.same_site_strict() {
        builder.field("same_site", &SameSite::Strict);
    } else if cookie.same_site_lax() {
        builder.field("same_site", &SameSite::Lax);
    }

    builder.finish()
}

pub(crate) fn display(cookie: &dyn Cookie, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(cookie.name())?;
    f.write_str("=")?;
    f.write_str(cookie.value())?;

    if let Some(path) = cookie.path() {
        f.write_str("; Path=")?;
        f.write_str(path)?;
    }

    if let Some(domain) = cookie.domain() {
        f.write_str("; Domain=")?;
        f.write_str(domain)?;
    }

    if let Some(ma) = cookie.max_age() {
        f.write_str("; Max-Age=")?;
        fmt::Display::fmt(&ma.as_secs(), f)?;

        // Include Expires, since some old user-agents don't support max-age
        let expires = get_expires(ma);
        f.write_str("; Expires=")?;
        fmt::Display::fmt(&expires.rfc822(), f)?;
    }

    if cookie.http_only() {
        f.write_str("; HttpOnly")?;
    }

    if cookie.secure() {
        f.write_str("; Secure")?;
    }

    if cookie.same_site_strict() {
        f.write_str("; SameSite=Strict")?;
    } else if cookie.same_site_lax() {
        f.write_str("; SameSite=Lax")?;
    }

    Ok(())
}

fn get_expires(dur: Duration) -> time::Tm {
    let t = if dur.as_secs() > std::i64::MAX as u64 {
        time::Timespec::new(std::i64::MAX, 0)
    } else {
        // Seconds since Unix Epoch...
        let mut t = time::get_time();
        // 'as_secs' within i64 thanks to check above
        // If the add would overflow, just assume the latest
        // possible time.
        t.sec = t.sec.saturating_add(dur.as_secs() as i64);
        t
    };
    time::at_utc(t)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn display_most_attributes() {
        let orig = "foo=bar; Path=/index.html; Domain=hyper.rs; HttpOnly; Secure; SameSite=Strict";
        let c = crate::parse(orig).unwrap();
        let s = c.to_string();
        assert_eq!(s, orig);
    }

    #[test]
    fn display_expires() {
        let c = crate::Builder::new("foo", "bar")
            .max_age(Duration::from_secs(100))
            .build()
            .unwrap();

        let s = c.to_string();

        let prefix = "foo=bar; Max-Age=100; Expires=";
        assert!(s.starts_with(prefix));
    }
}
