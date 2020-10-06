macro_rules! strings {
    ($($name:ident => $content:expr,)*) => (
        $(pub const $name: &str = $content;)*
    )
}

strings! {
    INPUT_EMAIL => "Email",
    INPUT_PASSWORD => "Password",
    TEXT_LOGIN => "Login",
    EMPTY_CONTRIBUTIONS => "This work does not have any contributions. Search contributors above to add its contributions.",
    EMPTY_ISSUES => "This work is not part of a series. Search above to add a new series issue.",
    EMPTY_PUBLICATIONS => "This work does not have any publications. Search above to add associated publications",
}
