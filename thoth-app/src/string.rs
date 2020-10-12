macro_rules! strings {
    ($($name:ident => $content:expr,)*) => (
        $(pub const $name: &str = $content;)*
    )
}

strings! {
    YES => "Yes",
    NO => "No",
    INPUT_EMAIL => "Email",
    INPUT_PASSWORD => "Password",
    TEXT_LOGIN => "Login",
    SAVE_BUTTON => "Save",
    REMOVE_BUTTON => "Remove",
    RELOAD_BUTTON => "Reload",
    NEXT_PAGE_BUTTON => "Next page",
    PREVIOUS_PAGE_BUTTON => "Previous",
    PAGINATION_COUNT_WORKS => "Displaying works",
    EMPTY_CONTRIBUTIONS => "This work does not have any contributions. Search contributors above to add its contributions.",
    EMPTY_ISSUES => "This work is not part of a series. Search above to add a new series issue.",
    EMPTY_LANGUAGES => "This work does not have any languages. Search above to add a new language.",
    EMPTY_PUBLICATIONS => "This work does not have any publications. Click above to add associated publications",
    EMPTY_SUBJECTS => "This work does not have any subjects. Click above to add associated subjects",
}
