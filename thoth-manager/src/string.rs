macro_rules! strings {
    ($($name:ident => $content:expr,)*) => (
        $(pub const $name: &str = $content;)*
    )
}

strings! {
    AUTHENTICATION_ERROR => "Authentication failed",
    INPUT_PASSWORD => "Password",
    INPUT_EMAIL => "Email",
    REQUEST_ERROR => "Failed to send request to server",
    RESPONSE_ERROR => "Failed to retrieve valid server response",
    TEXT_LOGIN => "Login",
}
