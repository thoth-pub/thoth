macro_rules! strings {
    ($($name:ident => $content:expr,)*) => (
        $(pub const $name: &str = $content;)*
    )
}

strings! {
    GRAPHQL_ENDPOINT => "http://localhost:8000/graphql",
}
