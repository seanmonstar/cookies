#![feature(test)]

extern crate test;

macro_rules! bench_parse {
    ($($name:ident: $source:expr,)+) => (
        mod old {
            use cookie::Cookie;

            $(
            #[bench]
            fn $name(b: &mut test::Bencher) {
                let src = $source;
                b.bytes = src.len() as u64;
                b.iter(|| {
                    let c = Cookie::parse(src).unwrap();
                    test::black_box(&c);
                });
            }
            )+
        }

        mod new {
            use cookies;

            $(
            #[bench]
            fn $name(b: &mut test::Bencher) {
                let src = $source;
                b.bytes = src.len() as u64;
                b.iter(|| {
                    let c = cookies::parse(src).unwrap();
                    test::black_box(&c);
                });
            }
            )+
        }
    )
}

bench_parse! {
    name_value: "hello=mynameiswat",
    expires: "hello=mynameiswat; Max-Age=100; Expires=Tue, 21 May 2019 21:12:11 GMT",
}
