#![feature(test)]

extern crate test;

static EXPECTED: &str = "hello=mynameiswat; Path=/; Domain=hyper.rs";

mod legacy {
    use super::*;
    use cookie::Cookie;

    #[bench]
    fn display_builder(b: &mut test::Bencher) {
        b.bytes = EXPECTED.len() as u64;
        let c = Cookie::build("hello", "mynameiswat")
            .path("/")
            .domain("hyper.rs")
            .finish();

        b.iter(|| {
            let s = c.to_string();
            assert_eq!(s, EXPECTED);
        });
    }
}

mod ng {
    use super::*;
    use cookies::Builder;

    #[bench]
    fn display_builder(b: &mut test::Bencher) {
        b.bytes = EXPECTED.len() as u64;
        let c = Builder::new("hello", "mynameiswat")
            .path("/")
            .domain("hyper.rs")
            .build()
            .unwrap();

        b.iter(|| {
            let s = c.to_string();
            assert_eq!(s, EXPECTED);
        });
    }
}
