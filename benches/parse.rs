#![feature(test)]

extern crate test;

static NAME_VALUE: &str = "hello=mynameiswat";
static WITH_EXPIRES: &str = "hello=mynameiswat; Max-Age=100; Expires=Tue, 21 May 2019 21:12:11 GMT";

mod legacy {
    use super::*;
    use cookie::Cookie;

    #[bench]
    fn parse_name_value(b: &mut test::Bencher) {
        b.bytes = NAME_VALUE.len() as u64;
        b.iter(|| {
            let c = Cookie::parse(NAME_VALUE).unwrap();
            test::black_box(&c);
        });
    }

    #[bench]
    fn parse_expires(b: &mut test::Bencher) {
        b.bytes = WITH_EXPIRES.len() as u64;
        b.iter(|| {
            let c = Cookie::parse(WITH_EXPIRES).unwrap();
            test::black_box(&c);
        });
    }
}

mod ng {
    use super::*;
    use cookies;

    #[bench]
    fn parse_name_value(b: &mut test::Bencher) {
        b.bytes = NAME_VALUE.len() as u64;
        b.iter(|| {
            let c = cookies::parse(NAME_VALUE).unwrap();
            test::black_box(&c);
        });
    }

    #[bench]
    fn parse_expires(b: &mut test::Bencher) {
        b.bytes = WITH_EXPIRES.len() as u64;
        b.iter(|| {
            let c = cookies::parse(WITH_EXPIRES).unwrap();
            test::black_box(&c);
        });
    }
}
