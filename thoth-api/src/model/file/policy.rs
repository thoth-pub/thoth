use super::{File, FileType, FileUpload, NewFile, NewFileUpload};
use crate::model::publication::PublicationType;
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext};
use thoth_errors::{ThothError, ThothResult};

const KIB: i64 = 1024;
const MIB: i64 = 1024 * 1024;
const GIB: i64 = 1024 * 1024 * 1024;
const MIN_PUBLICATION_BYTES: i64 = 50 * KIB;
const MAX_PUBLICATION_BYTES: i64 = 5 * GIB;
const MIN_FRONTCOVER_BYTES: i64 = 50 * KIB;
const MAX_FRONTCOVER_BYTES: i64 = 50 * MIB;

/// Write policies for `File` and `FileUpload`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring CDN write permissions scoped to the linked publisher
pub struct FilePolicy;

impl FilePolicy {
    fn normalize_mime_type(mime_type: &str) -> String {
        mime_type
            .split(';')
            .next()
            .unwrap_or(mime_type)
            .trim()
            .to_ascii_lowercase()
    }

    /// Validate file extension matches the file type and publication type (if applicable).
    pub(crate) fn validate_file_extension(
        extension: &str,
        file_type: &FileType,
        publication_type: Option<PublicationType>,
    ) -> ThothResult<()> {
        match file_type {
            FileType::Frontcover => {
                let valid_extensions = ["jpg", "jpeg", "png", "webp"];
                if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
                    return Err(ThothError::InvalidFileExtension);
                }
            }
            FileType::Publication => {
                if let Some(pub_type) = publication_type {
                    let valid_extensions: Vec<&str> = match pub_type {
                        // PDF
                        PublicationType::Pdf => vec!["pdf"],
                        // EPUB
                        PublicationType::Epub => vec!["epub"],
                        // HTML (including HTM and ZIP archives containing HTML)
                        PublicationType::Html => vec!["html", "htm", "zip"],
                        // XML (including ZIP archives containing XML)
                        PublicationType::Xml => vec!["xml", "zip"],
                        // DOCX
                        PublicationType::Docx => vec!["docx"],
                        // MOBI
                        PublicationType::Mobi => vec!["mobi"],
                        // AZW3
                        PublicationType::Azw3 => vec!["azw3"],
                        // FictionBook
                        PublicationType::FictionBook => vec!["fb2", "fb2.zip", "fbz", "zip"],
                        // MP3 audiobook
                        PublicationType::Mp3 => vec!["mp3"],
                        // WAV audiobook
                        PublicationType::Wav => vec!["wav"],
                        _ => return Err(ThothError::UnsupportedPublicationTypeForFileUpload),
                    };
                    if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
                        return Err(ThothError::InvalidFileExtension);
                    }
                } else {
                    return Err(ThothError::PublicationTypeRequiredForFileValidation);
                }
            }
        }
        Ok(())
    }

    /// Validate MIME type against file type/publication type allow-lists.
    pub(crate) fn validate_file_mime_type(
        extension: &str,
        file_type: &FileType,
        publication_type: Option<PublicationType>,
        mime_type: &str,
    ) -> ThothResult<()> {
        let mime_type = Self::normalize_mime_type(mime_type);
        match file_type {
            FileType::Frontcover => {
                let expected = match extension.to_ascii_lowercase().as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "webp" => "image/webp",
                    _ => return Err(ThothError::InvalidFileExtension),
                };

                if mime_type == expected {
                    Ok(())
                } else {
                    Err(ThothError::InvalidFileMimeType)
                }
            }
            FileType::Publication => {
                let publication_type =
                    publication_type.ok_or(ThothError::PublicationTypeRequiredForFileValidation)?;

                let accepted_mime_types: &[&str] = match publication_type {
                    PublicationType::Pdf => &["application/pdf", "application/octet-stream"],
                    PublicationType::Epub => &[
                        "application/epub+zip",
                        "application/zip",
                        "application/octet-stream",
                    ],
                    PublicationType::Html => {
                        &["text/html", "application/zip", "application/octet-stream"]
                    }
                    PublicationType::Xml => &[
                        "application/xml",
                        "text/xml",
                        "application/zip",
                        "application/octet-stream",
                    ],
                    PublicationType::Docx => &[
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                        "application/octet-stream",
                    ],
                    PublicationType::Mobi => {
                        &["application/x-mobipocket-ebook", "application/octet-stream"]
                    }
                    PublicationType::Azw3 => {
                        &["application/vnd.amazon.ebook", "application/octet-stream"]
                    }
                    PublicationType::FictionBook => &[
                        "application/fictionbook2+zip",
                        "application/zip",
                        "application/octet-stream",
                    ],
                    PublicationType::Mp3 => {
                        &["audio/mp3", "audio/mpeg", "application/octet-stream"]
                    }
                    PublicationType::Wav => {
                        &["audio/wav", "audio/x-wav", "application/octet-stream"]
                    }
                    _ => return Err(ThothError::UnsupportedPublicationTypeForFileUpload),
                };

                if accepted_mime_types.contains(&mime_type.as_str()) {
                    Ok(())
                } else {
                    Err(ThothError::InvalidFileMimeType)
                }
            }
        }
    }

    /// Validate uploaded object size limits.
    pub(crate) fn validate_file_size(bytes: i64, file_type: &FileType) -> ThothResult<()> {
        let (min_bytes, max_bytes) = match file_type {
            FileType::Publication => (MIN_PUBLICATION_BYTES, MAX_PUBLICATION_BYTES),
            FileType::Frontcover => (MIN_FRONTCOVER_BYTES, MAX_FRONTCOVER_BYTES),
        };

        if bytes < min_bytes {
            return Err(ThothError::FileTooSmall);
        }

        if bytes > max_bytes {
            return Err(ThothError::FileTooLarge);
        }

        Ok(())
    }

    /// Authorisation and validation gate for completing an upload.
    pub(crate) fn can_complete_upload<C: PolicyContext>(
        ctx: &C,
        upload: &FileUpload,
        publication_type: Option<PublicationType>,
        bytes: i64,
        mime_type: &str,
    ) -> ThothResult<()> {
        Self::can_delete(ctx, upload)?;
        Self::validate_file_extension(
            &upload.declared_extension,
            &upload.file_type,
            publication_type,
        )?;
        Self::validate_file_mime_type(
            &upload.declared_extension,
            &upload.file_type,
            publication_type,
            mime_type,
        )?;
        Self::validate_file_size(bytes, &upload.file_type)?;
        Ok(())
    }
}

impl CreatePolicy<NewFile> for FilePolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewFile, _params: ()) -> ThothResult<()> {
        ctx.require_cdn_write_for(data)?;
        Ok(())
    }
}

impl DeletePolicy<File> for FilePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, file: &File) -> ThothResult<()> {
        ctx.require_cdn_write_for(file)?;
        Ok(())
    }
}

impl CreatePolicy<NewFileUpload, Option<PublicationType>> for FilePolicy {
    fn can_create<C: PolicyContext>(
        ctx: &C,
        data: &NewFileUpload,
        publication_type: Option<PublicationType>,
    ) -> ThothResult<()> {
        ctx.require_cdn_write_for(data)?;
        Self::validate_file_extension(&data.declared_extension, &data.file_type, publication_type)?;
        Self::validate_file_mime_type(
            &data.declared_extension,
            &data.file_type,
            publication_type,
            &data.declared_mime_type,
        )?;
        Ok(())
    }
}

impl DeletePolicy<FileUpload> for FilePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, upload: &FileUpload) -> ThothResult<()> {
        ctx.require_cdn_write_for(upload)?;
        Ok(())
    }
}
