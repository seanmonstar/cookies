use cookies::Cookie;

macro_rules! t {
    (
        $fn:ident: $src:expr,
        $($method:ident = $value:expr,)+
    ) => (
        #[test]
        fn $fn() {
            let cookie = cookies::parse($src).expect("parse");

            // Check the getter methods
            $(
            assert_eq!(cookie.$method(), $value, "cookie.{}()", stringify!($method));
            )+
        }
    )
}

t! {
    name_val: "name=val",
    name = "name",
    value = "val",
}

t! {
    value_dollar: "Cookie-1=v$1",
    name = "Cookie-1",
    value = "v$1",
}

t! {
    name_dot: "ASP.NET_SessionId=foo; path=/; HttpOnly",
    name = "ASP.NET_SessionId",
    value = "foo",
    path = Some("/"),
    http_only = true,
}
