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
    DELETE_BUTTON => "Delete",
    CANCEL_BUTTON => "Cancel",
    REMOVE_BUTTON => "Remove",
    VIEW_BUTTON => "View",
    EDIT_BUTTON => "Edit",
    RELOAD_BUTTON => "Reload",
    NEXT_PAGE_BUTTON => "Next page",
    PREVIOUS_PAGE_BUTTON => "Previous",
    PAGINATION_COUNT_INSTITUTIONS => "Displaying institutions",
    PAGINATION_COUNT_WORKS => "Displaying works",
    PAGINATION_COUNT_BOOKS => "Displaying books",
    PAGINATION_COUNT_CHAPTERS => "Displaying chapters",
    PAGINATION_COUNT_SERIESES => "Displaying series",
    PAGINATION_COUNT_PUBLISHERS => "Displaying publishers",
    PAGINATION_COUNT_IMPRINTS => "Displaying imprints",
    PAGINATION_COUNT_CONTRIBUTORS => "Displaying contributors",
    PAGINATION_COUNT_PUBLICATIONS => "Displaying publications",
    AUTHENTICATION_ERROR => "Authentication failed",
    RESPONSE_ERROR => "Failed to obtain a valid response from the server.",
    EMPTY_RELATIONS => "This work does not have any related works. Search works above to add a new relation.",
    EMPTY_CONTRIBUTIONS => "This work does not have any contributions. Search contributors above to add its contributions.",
    EMPTY_ISSUES => "This work is not part of a series. Search above to add a new series issue.",
    EMPTY_LANGUAGES => "This work does not have any languages. Search above to add a new language.",
    EMPTY_LOCATIONS => "This publication does not have any location information. Click above to add locations.",
    EMPTY_PUBLICATIONS => "This work does not have any publications. Click above to add associated publications",
    EMPTY_SUBJECTS => "This work does not have any subjects. Click above to add associated subjects",
    EMPTY_FUNDINGS => "This work does not have any funding. Click above to add associated funding",
    EMPTY_PRICES => "This publication does not have any pricing information. Click above to add prices.",
    EMPTY_REFERENCES => "This work does not have any references. Click above to add references.",
    SEARCH_INSTITUTIONS => "Search by name, ROR ID or DOI",
    SEARCH_WORKS => "Search by title, DOI, internal reference, abstract or landing page",
    SEARCH_SERIESES => "Search by series name, ISSN, URL or description",
    SEARCH_PUBLISHERS => "Search by publisher name or short name",
    SEARCH_IMPRINTS => "Search by imprint name or URL",
    SEARCH_CONTRIBUTORS => "Search by name or ORCID",
    SEARCH_PUBLICATIONS => "Search by ISBN",
    STORAGE_ERROR => "local storage is disabled",
    PERMISSIONS_ERROR => "This account is not permitted to access any data",
    NEW_VERSION_PROMPT => "A new version of Thoth is available. Refresh your browser to update.",
    RELATIONS_INFO => "Relations below are saved automatically upon change.",
    NEW_CHAPTER_INFO => "Fields below are set automatically based on the current Work. Remaining fields are left as default.",
}
