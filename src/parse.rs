use std::fmt;
use std::time::Duration;

use super::{Cookie, Error, Sealed};
use crate::util::{self, SameSite};

const MAX_LENGTH: usize = 4096;

// Not:
// - PartialEq: determining equality depends on what you need equality for.
#[derive(Clone)]
struct Parsed<T> {
    src: T,

    name: Indexed,
    value: Indexed,
    domain: Option<Indexed>,
    path: Option<Indexed>,
    max_age: Option<Duration>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

// Cookie max length is 4kb, u16 can fit 64kb
type Indexed = (u16, u16);

fn indexed(s: &str, i: Indexed) -> &str {
    &s[i.0 as usize..i.1 as usize]
}

fn indices(src: &str, sub: &str) -> Indexed {
    debug_assert!(src.len() <= std::u16::MAX as usize);
    debug_assert!(sub.len() <= std::u16::MAX as usize);
    let start = sub.as_ptr() as usize - src.as_ptr() as usize;
    let end = start + sub.len();
    (start as u16, end as u16)
}

// ===== impl Parsed =====

impl<T: AsRef<str>> Cookie for Parsed<T> {
    fn name(&self) -> &str {
        indexed(self.src.as_ref(), self.name)
    }

    fn value(&self) -> &str {
        indexed(self.src.as_ref(), self.value)
    }

    fn domain(&self) -> Option<&str> {
        self.domain.map(|i| indexed(self.src.as_ref(), i))
    }

    fn path(&self) -> Option<&str> {
        self.path.map(|i| indexed(self.src.as_ref(), i))
    }

    fn max_age(&self) -> Option<Duration> {
        self.max_age
    }

    fn http_only(&self) -> bool {
        self.http_only
    }

    fn secure(&self) -> bool {
        self.secure
    }

    fn same_site(&self) -> Option<SameSite> {
        self.same_site
    }
}

impl<T: AsRef<str>> Sealed for Parsed<T> {}

/* TODO?
impl<'t, T: AsRef<str>> Parsed<&'t T> {
    pub fn value_ref(&self) -> &'t str {
        indexed(self.src.as_ref(), self.value)
    }
}
*/

impl<T: AsRef<str>> fmt::Debug for Parsed<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        util::debug(self, f)
    }
}

impl<T: AsRef<str>> fmt::Display for Parsed<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        util::display(self, f)
    }
}

/// Parse some string as a `Cookie`.
///
/// # Example
///
/// ```
/// use cookies::Cookie;
///
/// let raw = "foo=bar; Domain=hyper.rs";
///
/// let cookie = cookies::parse(raw).expect("parse error");
///
/// assert_eq!(cookie.name(), "foo");
/// assert_eq!(cookie.value(), "bar");
/// assert_eq!(cookie.domain(), Some("hyper.rs"));
/// ```
pub fn parse<T: AsRef<str>>(src: T) -> Result<impl Cookie, Error> {
    let mut cookie = Parsed {
        src,
        name: (0, 0),
        value: (0, 0),
        domain: None,
        path: None,
        max_age: None,
        http_only: false,
        secure: false,
        same_site: None,
    };

    let s = cookie.src.as_ref();

    if s.len() > MAX_LENGTH {
        return Err(Error::too_long());
    }

    let mut attrs = s.split(';');

    let name_value = attrs.next().expect("split always has at least 1 item");

    match name_value.find('=') {
        Some(i) => {
            let n = name_value[..i].trim();
            validate_name(n)?;
            let v = name_value[(i + 1)..].trim();
            validate_value(v)?;
            cookie.name = indices(s, n);
            cookie.value = indices(s, v);
        }
        None => return Err(Error::invalid_name()),
    }

    // A lazy `Expires` attribute, since `Max-Age` takes precedence, we can
    // skip parsing the date if a `Max-Age` was included as well.
    let mut expires = None;

    for attr in attrs {
        let (name, value) = match attr.find('=') {
            Some(i) => (attr[..i].trim(), Some(attr[(i + 1)..].trim())),
            None => (attr.trim(), None),
        };

        if name.eq_ignore_ascii_case("secure") {
            cookie.secure = true;
        } else if name.eq_ignore_ascii_case("httponly") {
            cookie.http_only = true;
        } else if let Some(value) = value {
            if name.eq_ignore_ascii_case("max-age") {
                cookie.max_age = match value.parse::<i64>() {
                    Ok(secs) if secs <= 0 => Some(Duration::from_secs(0)),
                    Ok(secs) => Some(Duration::from_secs(secs as u64)),
                    Err(_) => {
                        // Don't change `cookie.max_age` otherwise, a previous
                        // attribute may have been valid.
                        //
                        // This case is checked in unit tests below.
                        continue;
                    }
                };
            } else if name.eq_ignore_ascii_case("path") {
                if !is_valid_path(value) {
                    continue;
                }
                cookie.path = Some(indices(s, value));
            } else if name.eq_ignore_ascii_case("domain") {
                cookie.domain = match validate_domain(value) {
                    Domain::AsIs => Some(indices(s, value)),
                    Domain::LeadingDot => Some(indices(s, &value[1..])),
                    Domain::Invalid => continue,
                }
            } else if name.eq_ignore_ascii_case("expires") {
                expires = Some(value);
            } else if name.eq_ignore_ascii_case("samesite") {
                cookie.same_site = if value.eq_ignore_ascii_case("lax") {
                    Some(SameSite::LAX)
                } else if value.eq_ignore_ascii_case("strict") {
                    Some(SameSite::STRICT)
                } else {
                    // unknown SameSite, skip as mandated by spec
                    continue;
                }
            } else {
                // ignoring unknown attribute, as mandated by RFC6265
            }
        }
    }

    if let (Some(expires), None) = (expires, cookie.max_age) {
        let tm = time::strptime(expires, "%a, %d %b %Y %T %Z")
            .or_else(|_| time::strptime(expires, "%A, %d-%b-%y %T %Z"))
            .or_else(|_| time::strptime(expires, "%c"));

        if let Ok(tm) = tm {
            let expires_tspec = tm.to_timespec();
            let now = time::get_time();
            if expires_tspec.sec > now.sec && expires_tspec.sec > 0 {
                // as u64: Just checked the values are positive
                let secs = (expires_tspec.sec - now.sec) as u64;
                cookie.max_age = Some(Duration::from_secs(secs));
            } else {
                // already expired
                cookie.max_age = Some(Duration::from_secs(0));
            }
        }
    }

    Ok(cookie)
}

pub(crate) fn validate_name(n: &str) -> Result<(), Error> {
    if n.is_empty() {
        return Err(Error::invalid_name());
    }
    // token = 1*<any CHAR except CTLs or separators>
    // separators = "(" | ")" | "<" | ">" | "@"
    // | "," | ";" | ":" | "\" | <">
    // | "/" | "[" | "]" | "?" | "="
    // | "{" | "}" | SP | HT
    for &byte in n.as_bytes() {
        match byte {
            b'('
            | b')'
            | b'<'
            | b'>'
            | b'@'
            | b','
            | b';'
            | b':'
            | b'\\'
            | b'"'
            | b'/'
            | b'['
            | b']'
            | b'?'
            | b'='
            | b'{'
            | b'}'
            | b' '
            | b'\t'
            | 0..=32
            | 127 => return Err(Error::invalid_name()),
            _ => (),
        }
    }
    Ok(())
}

pub(crate) fn validate_value(v: &str) -> Result<(), Error> {
    // cookie-octet = %x21 / %x23-2B / %x2D-3A / %x3C-5B / %x5D-7E
    // US-ASCII characters excluding CTLs, whitespace, DQUOTE, comma, semicolon,
    // and backslash

    for &byte in v.as_bytes() {
        match byte {
            0x21 | 0x23..=0x2B | 0x2D..=0x3A | 0x3C..=0x5B | 0x5D..=0x7E => (),
            _ => return Err(Error::invalid_value()),
        }
    }

    Ok(())
}

pub(crate) fn is_valid_path(p: &str) -> bool {
    if p.is_empty() {
        return false;
    }

    if p.as_bytes()[0] != b'/' {
        return false;
    }

    // prevent CTL characters because sanity
    for &byte in p.as_bytes() {
        match byte {
            0..=32 | b';' | 127 => return false,
            _ => (),
        }
    }

    true
}

pub(crate) enum Domain {
    AsIs,
    LeadingDot,
    Invalid,
}

pub(crate) fn validate_domain(d: &str) -> Domain {
    if d.is_empty() {
        return Domain::Invalid;
    }

    // prevent CTL characters because sanity
    for &byte in d.as_bytes() {
        match byte {
            0..=32 | b';' | 127 => return Domain::Invalid,
            _ => (),
        }
    }

    // > If the first character of the attribute-value string is
    // > %x2E ("."):
    // >
    // >   Let cookie-domain be the attribute-value without the
    // >   leading %x2E (".") character.
    //
    // https://tools.ietf.org/html/rfc6265#section-5.2.3
    if d.as_bytes()[0] == b'.' {
        Domain::LeadingDot
    } else {
        Domain::AsIs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_name() {
        parse("=bar").expect_err("empty name");
        parse("f()()=bar").expect_err("parens in name");
    }

    #[test]
    fn invalid_value() {
        parse("foo=b\na\nr").expect_err("CTL in value");
    }

    #[test]
    fn max_age() {
        let secs_3 = Duration::from_secs(3);

        let c = parse("foo=bar; Max-Age=3").expect("positive");
        assert_eq!(c.max_age(), Some(secs_3));

        let c = parse("foo=bar; Max-Age=-3").expect("negative");
        assert_eq!(c.max_age(), Some(Duration::from_secs(0)));

        let c = parse("foo=bar; Max-Age=55; Max-Age=3").unwrap();
        assert_eq!(c.max_age(), Some(secs_3), "last Max-Age");

        let c = parse("foo=bar; Max-Age=3; Max-Age=wat").unwrap();
        assert_eq!(c.max_age(), Some(secs_3), "last 'valid' Max-Age");
    }

    #[test]
    fn path() {
        let c = parse("foo=bar; Path=/").unwrap();
        assert_eq!(c.path(), Some("/"));

        let c = parse("foo=bar; Path=/next").unwrap();
        assert_eq!(c.path(), Some("/next"));

        let c = parse("foo=bar; Path").unwrap();
        assert_eq!(c.path(), None, "Path without equals");

        let c = parse("foo=bar; Path=").unwrap();
        assert_eq!(c.path(), None, "Path with empty value");

        let c = parse("foo=bar; Path=/a; Path=/b").unwrap();
        assert_eq!(c.path(), Some("/b"), "is last Path");

        let c = parse("foo=bar; Path=/a; Path=").unwrap();
        assert_eq!(c.path(), Some("/a"), "is last 'valid' Path");

        let c = parse("foo=bar; Path=woop/sies").unwrap();
        assert_eq!(c.path(), None, "first character is non-slash");
    }

    #[test]
    fn domain() {
        let c = parse("foo=bar; Domain=hyper.rs").unwrap();
        assert_eq!(c.domain(), Some("hyper.rs"));

        let c = parse("foo=bar; Domain=.hyper.rs").unwrap();
        assert_eq!(c.domain(), Some("hyper.rs"), "removes leading dot");

        let c = parse("foo=bar; Domain=hyper.rs; Domain=rust-lang.org").unwrap();
        assert_eq!(c.domain(), Some("rust-lang.org"), "is last Domain");

        let c = parse("foo=bar; Domain=hyper.rs; Domain").unwrap();
        assert_eq!(c.domain(), Some("hyper.rs"), "is last 'valid' Domain");
    }

    #[test]
    fn secure_bogus_value() {
        let c = parse("foo=bar; secure=wat").unwrap();
        assert!(c.secure());
    }

    #[test]
    fn httponly_bogus_value() {
        let c = parse("foo=bar; httponly=wat").unwrap();
        assert!(c.http_only());
    }

    #[test]
    fn samesite_bogus_value() {
        // SameSite spec says we should ignore the attribute completely
        let c = parse("foo=bar; samesite=wat").unwrap();
        assert_eq!(c.same_site(), None);
    }

    #[test]
    fn parsed_to_boxed() {
        let c = parse("foo=bar").unwrap();
        let bc = Box::new(c) as Box<dyn Cookie>;
        assert_eq!(bc.name(), "foo");
        assert_eq!(bc.value(), "bar");
    }
}
