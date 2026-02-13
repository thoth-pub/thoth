use super::{File, FileType, FileUpload, NewFile, NewFileUpload};
use crate::model::publication::PublicationType;
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `File` and `FileUpload`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring CDN write permissions scoped to the linked publisher
pub struct FilePolicy;

impl FilePolicy {
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
                        PublicationType::FictionBook => vec!["fb2", "fbz", "zip"],
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

    /// Authorisation and validation gate for completing an upload.
    pub(crate) fn can_complete_upload<C: PolicyContext>(
        ctx: &C,
        upload: &FileUpload,
        publication_type: Option<PublicationType>,
    ) -> ThothResult<()> {
        Self::can_delete(ctx, upload)?;
        Self::validate_file_extension(
            &upload.declared_extension,
            &upload.file_type,
            publication_type,
        )
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
        Self::validate_file_extension(&data.declared_extension, &data.file_type, publication_type)
    }
}

impl DeletePolicy<FileUpload> for FilePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, upload: &FileUpload) -> ThothResult<()> {
        ctx.require_cdn_write_for(upload)?;
        Ok(())
    }
}
