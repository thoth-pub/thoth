use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

pub(crate) trait BibtexSpecification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut buffer: Vec<u8> = Vec::new();
        Self::handle_event(&mut buffer, works)
            .map(|_| buffer)
            .and_then(|bibtex| {
                String::from_utf8(bibtex)
                    .map_err(|_| ThothError::InternalError("Could not parse BibTeX".to_string()))
            })
    }

    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait BibtexEntry<T: BibtexSpecification> {
    fn bibtex_entry(&self, w: &mut Vec<u8>) -> ThothResult<()>;
}

/// Macro to write a non-optional field in BibTeX format to a formatter.
///
/// This macro writes a non-optional field of a struct to a formatter in BibTeX format,
/// including the field name and its value. If a custom field name is not provided,
/// the stringified version of the field name will be used.
///
/// By default, the value will be enclosed in braces `{}`, which is intended to be used
/// with `String`. By passing a type onto the macro we instruct it  not to enclose
/// the value, which is intended to be used with `i64`.
///
/// # Examples
///
/// ```
/// # use std::fmt::Write;
/// # use thoth_export_server::write_field;
/// # struct BibtexThothEntry {
/// #     title: String,
/// #     year: i64,
/// # }
/// # fn run() -> Result<(), std::fmt::Error> {
/// # let mut f = String::new();
/// # let entry = BibtexThothEntry { title: "Example Title".to_string(), year: 2024 };
/// write_field!(f, entry, title);
/// assert_eq!(f, ",\n\ttitle = {Example Title}");
/// f.clear();
///
/// write_field!(f, entry, title, "abbr_title");
/// assert_eq!(f, ",\n\tabbr_title = {Example Title}");
/// f.clear();
///
/// write_field!(f, entry, year, i64);
/// assert_eq!(f, ",\n\tyear = 2024");
/// f.clear();
///
/// write_field!(f, entry, year, "date", i64);
/// assert_eq!(f, ",\n\tdate = 2024");
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! write_field {
    ($f:ident, $self:ident, $field:ident) => {
        write!($f, ",\n\t{} = {{{}}}", stringify!($field), $self.$field)?;
    };
    ($f:ident, $self:ident, $field:ident, $t:ty) => {
        write!($f, ",\n\t{} = {}", stringify!($field), $self.$field)?;
    };
    ($f:ident, $self:ident, $field:ident, $field_name:expr) => {
        write!($f, ",\n\t{} = {{{}}}", $field_name, $self.$field)?;
    };
    ($f:ident, $self:ident, $field:ident, $field_name:expr, $t:ty) => {
        write!($f, ",\n\t{} = {}", $field_name, $self.$field)?;
    };
}

/// Macro to write an optional field in BibTeX format to a formatter.
///
/// This macro writes an optional field of a struct to a formatter in BibTeX format,
/// including the field name and its value, only if the field has a value.
/// If a custom field name is not provided, the stringified version of the field
/// name will be used.
///
/// By default, the value will be enclosed in braces `{}`, which is intended to be used
/// with `Option<String>`. By passing a type onto the macro we instruct it
/// not to enclose the value, which is intended to be used with `Option<i64>`.
///
/// # Examples
///
/// ```
/// # use std::fmt::Write;
/// # use thoth_export_server::write_optional_field;
/// # struct BibtexThothEntry {
/// #     title: Option<String>,
/// #     year: Option<i64>,
/// # }
/// # fn run() -> Result<(), std::fmt::Error> {
/// # let mut f = String::new();
/// # let entry = BibtexThothEntry { title: Some("Example Title".to_string()), year: Some(2024) };
/// write_optional_field!(f, entry, title);
/// assert_eq!(f, ",\n\ttitle = {Example Title}");
/// f.clear();
///
/// write_optional_field!(f, entry, title, "abbr_title");
/// assert_eq!(f, ",\n\tabbr_title = {Example Title}");
/// f.clear();
///
/// write_optional_field!(f, entry, year, i64);
/// assert_eq!(f, ",\n\tyear = 2024");
/// f.clear();
///
/// write_optional_field!(f, entry, year, "date", i64);
/// assert_eq!(f, ",\n\tdate = 2024");
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! write_optional_field {
    ($f:ident, $self:ident, $field:ident) => {
        if let Some(value) = &$self.$field {
            write!($f, ",\n\t{} = {{{}}}", stringify!($field), value)?;
        }
    };
    ($f:ident, $self:ident, $field:ident, $t:ty) => {
        if let Some(value) = &$self.$field {
            write!($f, ",\n\t{} = {}", stringify!($field), value)?;
        }
    };
    ($f:ident, $self:ident, $field:ident, $field_name:expr) => {
        if let Some(value) = &$self.$field {
            write!($f, ",\n\t{} = {{{}}}", $field_name, value)?;
        }
    };
    ($f:ident, $self:ident, $field:ident, $field_name:expr, $t:ty) => {
        if let Some(value) = &$self.$field {
            write!($f, ",\n\t{} = {}", $field_name, value)?;
        }
    };
}

mod bibtex_thoth;
pub(crate) use bibtex_thoth::BibtexThoth;
