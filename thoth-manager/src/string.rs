macro_rules! strings {
    ($($name:ident => $content:expr,)*) => (
        $(pub const $name: &str = $content;)*
    )
}

strings! {
    INPUT_EMAIL => "Email",
    INPUT_PASSWORD => "Password",
    TEXT_LOGIN => "Login",
}
