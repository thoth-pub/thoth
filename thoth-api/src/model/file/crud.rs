use super::FileType;
use super::{
    upload_request_headers, File, FileCleanupCandidate, FilePolicy, FileUpload, FileUploadResponse,
    NewFile, NewFileUpload,
};
use crate::db::PgPool;
use crate::model::{
    additional_resource::{AdditionalResource, PatchAdditionalResource},
    location::{Location, LocationPlatform, NewLocation, PatchLocation},
    publication::Publication,
    work::{PatchWork, Work},
    work_featured_video::{PatchWorkFeaturedVideo, WorkFeaturedVideo},
    Crud, Doi, PublisherId, Timestamp,
};
use crate::policy::{CreatePolicy, PolicyContext};
use crate::schema::{file, file_upload};
use crate::storage::{
    canonical_frontcover_key, canonical_publication_key, canonical_resource_key,
    presign_put_for_upload, temp_key, S3Client, StorageConfig,
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::OptionalExtension;
use std::collections::HashSet;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

type FileUploadScope = (
    Work,
    Option<Publication>,
    Option<AdditionalResource>,
    Option<WorkFeaturedVideo>,
);

fn upload_expires_at(minutes: i64) -> ThothResult<Timestamp> {
    let expires_at = Utc::now()
        .checked_add_signed(Duration::minutes(minutes))
        .ok_or_else(|| {
            ThothError::InternalError("Failed to calculate expiration time".to_string())
        })?;
    Timestamp::parse_from_rfc3339(&expires_at.to_rfc3339())
}

fn publisher_id_from_scope(
    db: &PgPool,
    work_id: Option<Uuid>,
    publication_id: Option<Uuid>,
    additional_resource_id: Option<Uuid>,
    work_featured_video_id: Option<Uuid>,
    missing_scope_error: ThothError,
) -> ThothResult<Uuid> {
    match (
        work_id,
        publication_id,
        additional_resource_id,
        work_featured_video_id,
    ) {
        (Some(work_id), None, None, None) => Work::from_id(db, &work_id)?.publisher_id(db),
        (None, Some(publication_id), None, None) => {
            Publication::from_id(db, &publication_id)?.publisher_id(db)
        }
        (None, None, Some(additional_resource_id), None) => {
            AdditionalResource::from_id(db, &additional_resource_id)?.publisher_id(db)
        }
        (None, None, None, Some(work_featured_video_id)) => {
            WorkFeaturedVideo::from_id(db, &work_featured_video_id)?.publisher_id(db)
        }
        _ => Err(missing_scope_error),
    }
}

impl Crud for File {
    type NewEntity = NewFile;
    type PatchEntity = NewFile;
    type OrderByEntity = ();
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.file_id
    }

    fn all(
        _db: &PgPool,
        _limit: i32,
        _offset: i32,
        _filter: Option<String>,
        _order: Self::OrderByEntity,
        _publishers: Vec<Uuid>,
        _parent_id_1: Option<Uuid>,
        _parent_id_2: Option<Uuid>,
        _filter_param_1: Vec<Self::FilterParameter1>,
        _filter_param_2: Vec<Self::FilterParameter2>,
        _filter_param_3: Option<Self::FilterParameter3>,
        _filter_param_4: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<File>> {
        unimplemented!()
    }

    fn count(
        _db: &PgPool,
        _filter: Option<String>,
        _publishers: Vec<Uuid>,
        _filter_param_1: Vec<Self::FilterParameter1>,
        _filter_param_2: Vec<Self::FilterParameter2>,
        _filter_param_3: Option<Self::FilterParameter3>,
        _filter_param_4: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        unimplemented!()
    }

    fn from_id(db: &PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        let mut connection = db.get()?;
        file::table
            .find(entity_id)
            .get_result::<File>(&mut connection)
            .map_err(ThothError::from)
    }

    fn create(db: &PgPool, data: &NewFile) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::insert_into(file::table)
            .values(data)
            .get_result::<File>(&mut connection)
            .map_err(ThothError::from)
    }

    fn update<C: PolicyContext>(&self, _ctx: &C, _data: &NewFile) -> ThothResult<Self> {
        unimplemented!()
    }

    fn delete(self, db: &PgPool) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::delete(file::table.find(self.file_id))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(ThothError::from)
    }
}

impl Crud for FileUpload {
    type NewEntity = NewFileUpload;
    type PatchEntity = NewFileUpload;
    type OrderByEntity = ();
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.file_upload_id
    }

    fn all(
        _db: &PgPool,
        _limit: i32,
        _offset: i32,
        _filter: Option<String>,
        _order: Self::OrderByEntity,
        _publishers: Vec<Uuid>,
        _parent_id_1: Option<Uuid>,
        _parent_id_2: Option<Uuid>,
        _filter_param_1: Vec<Self::FilterParameter1>,
        _filter_param_2: Vec<Self::FilterParameter2>,
        _filter_param_3: Option<Self::FilterParameter3>,
        _filter_param_4: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<FileUpload>> {
        unimplemented!()
    }

    fn count(
        _db: &PgPool,
        _filter: Option<String>,
        _publishers: Vec<Uuid>,
        _filter_param_1: Vec<Self::FilterParameter1>,
        _filter_param_2: Vec<Self::FilterParameter2>,
        _filter_param_3: Option<Self::FilterParameter3>,
        _filter_param_4: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        unimplemented!()
    }

    fn from_id(db: &PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        let mut connection = db.get()?;
        file_upload::table
            .find(entity_id)
            .get_result::<FileUpload>(&mut connection)
            .map_err(ThothError::from)
    }

    fn create(db: &PgPool, data: &NewFileUpload) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::insert_into(file_upload::table)
            .values(data)
            .get_result::<FileUpload>(&mut connection)
            .map_err(ThothError::from)
    }

    fn update<C: PolicyContext>(&self, _ctx: &C, _data: &NewFileUpload) -> ThothResult<Self> {
        unimplemented!()
    }

    fn delete(self, db: &PgPool) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::delete(file_upload::table.find(self.file_upload_id))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(ThothError::from)
    }
}

impl PublisherId for File {
    fn publisher_id(&self, db: &PgPool) -> ThothResult<Uuid> {
        publisher_id_from_scope(
            db,
            self.work_id,
            self.publication_id,
            self.additional_resource_id,
            self.work_featured_video_id,
            ThothError::FileMissingWorkOrPublicationId,
        )
    }
}

impl PublisherId for NewFile {
    fn publisher_id(&self, db: &PgPool) -> ThothResult<Uuid> {
        publisher_id_from_scope(
            db,
            self.work_id,
            self.publication_id,
            self.additional_resource_id,
            self.work_featured_video_id,
            ThothError::FileMissingWorkOrPublicationId,
        )
    }
}

impl PublisherId for FileUpload {
    fn publisher_id(&self, db: &PgPool) -> ThothResult<Uuid> {
        publisher_id_from_scope(
            db,
            self.work_id,
            self.publication_id,
            self.additional_resource_id,
            self.work_featured_video_id,
            ThothError::FileUploadMissingWorkOrPublicationId,
        )
    }
}

impl PublisherId for NewFileUpload {
    fn publisher_id(&self, db: &PgPool) -> ThothResult<Uuid> {
        publisher_id_from_scope(
            db,
            self.work_id,
            self.publication_id,
            self.additional_resource_id,
            self.work_featured_video_id,
            ThothError::FileUploadMissingWorkOrPublicationId,
        )
    }
}

impl File {
    fn deduplicate_cleanup_candidates(
        candidates: Vec<FileCleanupCandidate>,
    ) -> Vec<FileCleanupCandidate> {
        let mut seen = HashSet::new();
        let mut deduplicated = Vec::new();

        for candidate in candidates {
            if seen.insert(candidate.object_key.clone()) {
                deduplicated.push(candidate);
            }
        }

        deduplicated.sort_by(|a, b| a.object_key.cmp(&b.object_key));
        deduplicated
    }

    fn to_cleanup_candidates(files: Vec<File>) -> Vec<FileCleanupCandidate> {
        files
            .into_iter()
            .map(|file| FileCleanupCandidate {
                file_type: file.file_type,
                object_key: file.object_key,
            })
            .collect()
    }

    pub fn cleanup_candidates_for_publication(
        db: &PgPool,
        publication_id: &Uuid,
    ) -> ThothResult<Vec<FileCleanupCandidate>> {
        use crate::schema::file::dsl as file_dsl;

        let mut connection = db.get()?;
        let files = file_dsl::file
            .filter(file_dsl::publication_id.eq(Some(*publication_id)))
            .load::<File>(&mut connection)
            .map_err(ThothError::from)?;

        Ok(Self::deduplicate_cleanup_candidates(
            Self::to_cleanup_candidates(files),
        ))
    }

    pub fn cleanup_candidates_for_work(
        db: &PgPool,
        work_id: &Uuid,
    ) -> ThothResult<Vec<FileCleanupCandidate>> {
        use crate::schema::additional_resource::dsl as additional_resource_dsl;
        use crate::schema::publication::dsl as publication_dsl;
        use crate::schema::work_featured_video::dsl as work_featured_video_dsl;

        let mut connection = db.get()?;

        let publication_ids = publication_dsl::publication
            .filter(publication_dsl::work_id.eq(*work_id))
            .select(publication_dsl::publication_id)
            .load::<Uuid>(&mut connection)
            .map_err(ThothError::from)?;

        let additional_resource_ids = additional_resource_dsl::additional_resource
            .filter(additional_resource_dsl::work_id.eq(*work_id))
            .select(additional_resource_dsl::additional_resource_id)
            .load::<Uuid>(&mut connection)
            .map_err(ThothError::from)?;

        let work_featured_video_ids = work_featured_video_dsl::work_featured_video
            .filter(work_featured_video_dsl::work_id.eq(*work_id))
            .select(work_featured_video_dsl::work_featured_video_id)
            .load::<Uuid>(&mut connection)
            .map_err(ThothError::from)?;

        let mut candidates = Vec::new();

        let direct_work_files = crate::schema::file::dsl::file
            .filter(crate::schema::file::dsl::work_id.eq(Some(*work_id)))
            .load::<File>(&mut connection)
            .map_err(ThothError::from)?;
        candidates.extend(Self::to_cleanup_candidates(direct_work_files));

        if !publication_ids.is_empty() {
            let publication_files = crate::schema::file::dsl::file
                .filter(crate::schema::file::dsl::publication_id.eq_any(&publication_ids))
                .load::<File>(&mut connection)
                .map_err(ThothError::from)?;
            candidates.extend(Self::to_cleanup_candidates(publication_files));
        }

        if !additional_resource_ids.is_empty() {
            let additional_resource_files = crate::schema::file::dsl::file
                .filter(
                    crate::schema::file::dsl::additional_resource_id
                        .eq_any(&additional_resource_ids),
                )
                .load::<File>(&mut connection)
                .map_err(ThothError::from)?;
            candidates.extend(Self::to_cleanup_candidates(additional_resource_files));
        }

        if !work_featured_video_ids.is_empty() {
            let work_featured_video_files = crate::schema::file::dsl::file
                .filter(
                    crate::schema::file::dsl::work_featured_video_id
                        .eq_any(&work_featured_video_ids),
                )
                .load::<File>(&mut connection)
                .map_err(ThothError::from)?;
            candidates.extend(Self::to_cleanup_candidates(work_featured_video_files));
        }

        Ok(Self::deduplicate_cleanup_candidates(candidates))
    }

    pub fn from_object_key(db: &PgPool, object_key: &str) -> ThothResult<Self> {
        use crate::schema::file::dsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::object_key.eq(object_key))
            .first::<File>(&mut connection)
            .map_err(ThothError::from)
    }

    pub fn from_work_id(db: &PgPool, work_id: &Uuid) -> ThothResult<Option<Self>> {
        use crate::schema::file::dsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::work_id.eq(work_id))
            .filter(dsl::file_type.eq(FileType::Frontcover))
            .first::<File>(&mut connection)
            .optional()
            .map_err(ThothError::from)
    }

    pub fn from_publication_id(db: &PgPool, publication_id: &Uuid) -> ThothResult<Option<Self>> {
        use crate::schema::file::dsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::publication_id.eq(publication_id))
            .filter(dsl::file_type.eq(FileType::Publication))
            .first::<File>(&mut connection)
            .optional()
            .map_err(ThothError::from)
    }

    pub fn from_additional_resource_id(
        db: &PgPool,
        additional_resource_id: &Uuid,
    ) -> ThothResult<Option<Self>> {
        use crate::schema::file::dsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::additional_resource_id.eq(additional_resource_id))
            .filter(dsl::file_type.eq(FileType::AdditionalResource))
            .first::<File>(&mut connection)
            .optional()
            .map_err(ThothError::from)
    }

    pub fn from_work_featured_video_id(
        db: &PgPool,
        work_featured_video_id: &Uuid,
    ) -> ThothResult<Option<Self>> {
        use crate::schema::file::dsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::work_featured_video_id.eq(work_featured_video_id))
            .filter(dsl::file_type.eq(FileType::WorkFeaturedVideo))
            .first::<File>(&mut connection)
            .optional()
            .map_err(ThothError::from)
    }
}

impl NewFileUpload {
    pub(crate) async fn create_upload_response(
        &self,
        db: &PgPool,
        s3_client: &S3Client,
        storage_config: &StorageConfig,
        expires_in_minutes: u64,
    ) -> ThothResult<FileUploadResponse> {
        let file_upload = FileUpload::create(db, self)?;
        let temp_object_key = temp_key(&file_upload.file_upload_id);
        let upload_url = presign_put_for_upload(
            s3_client,
            &storage_config.s3_bucket,
            &temp_object_key,
            &self.declared_mime_type,
            &self.declared_sha256,
            expires_in_minutes,
        )
        .await?;

        let upload_headers =
            upload_request_headers(&self.declared_mime_type, &self.declared_sha256)?;

        Ok(FileUploadResponse {
            file_upload_id: file_upload.file_upload_id,
            upload_url,
            upload_headers,
            expires_at: upload_expires_at(expires_in_minutes as i64)?,
        })
    }
}

impl FileUpload {
    pub(crate) fn load_scope<C: PolicyContext>(&self, ctx: &C) -> ThothResult<FileUploadScope> {
        match self.file_type {
            FileType::Publication => {
                let publication_id = self
                    .publication_id
                    .ok_or(ThothError::PublicationFileUploadMissingPublicationId)?;
                let publication: Publication = ctx.load_current(&publication_id)?;
                let work: Work = ctx.load_current(&publication.work_id)?;
                Ok((work, Some(publication), None, None))
            }
            FileType::Frontcover => {
                let work_id = self
                    .work_id
                    .ok_or(ThothError::FrontcoverFileUploadMissingWorkId)?;
                let work: Work = ctx.load_current(&work_id)?;
                Ok((work, None, None, None))
            }
            FileType::AdditionalResource => {
                let additional_resource_id = self
                    .additional_resource_id
                    .ok_or(ThothError::AdditionalResourceFileUploadMissingAdditionalResourceId)?;
                let additional_resource: AdditionalResource =
                    ctx.load_current(&additional_resource_id)?;
                let work: Work = ctx.load_current(&additional_resource.work_id)?;
                Ok((work, None, Some(additional_resource), None))
            }
            FileType::WorkFeaturedVideo => {
                let work_featured_video_id = self
                    .work_featured_video_id
                    .ok_or(ThothError::WorkFeaturedVideoFileUploadMissingWorkFeaturedVideoId)?;
                let work_featured_video: WorkFeaturedVideo =
                    ctx.load_current(&work_featured_video_id)?;
                let work: Work = ctx.load_current(&work_featured_video.work_id)?;
                Ok((work, None, None, Some(work_featured_video)))
            }
        }
    }

    pub(crate) fn canonical_key(&self, doi: &Doi) -> ThothResult<String> {
        let doi_prefix = doi.prefix();
        let doi_suffix = doi.suffix();

        match self.file_type {
            FileType::Publication => Ok(canonical_publication_key(
                doi_prefix,
                doi_suffix,
                &self.declared_extension,
            )),
            FileType::Frontcover => Ok(canonical_frontcover_key(
                doi_prefix,
                doi_suffix,
                &self.declared_extension,
            )),
            FileType::AdditionalResource => {
                let additional_resource_id = self
                    .additional_resource_id
                    .ok_or(ThothError::AdditionalResourceFileUploadMissingAdditionalResourceId);
                Ok(canonical_resource_key(
                    doi_prefix,
                    doi_suffix,
                    &additional_resource_id?,
                    &self.declared_extension,
                ))
            }
            FileType::WorkFeaturedVideo => {
                let work_featured_video_id = self
                    .work_featured_video_id
                    .ok_or(ThothError::WorkFeaturedVideoFileUploadMissingWorkFeaturedVideoId);
                Ok(canonical_resource_key(
                    doi_prefix,
                    doi_suffix,
                    &work_featured_video_id?,
                    &self.declared_extension,
                ))
            }
        }
    }

    pub(crate) fn existing_file(&self, db: &PgPool) -> ThothResult<Option<File>> {
        match self.file_type {
            FileType::Publication => {
                let publication_id = self
                    .publication_id
                    .ok_or(ThothError::PublicationFileUploadMissingPublicationId)?;
                File::from_publication_id(db, &publication_id)
            }
            FileType::Frontcover => {
                let work_id = self
                    .work_id
                    .ok_or(ThothError::FrontcoverFileUploadMissingWorkId)?;
                File::from_work_id(db, &work_id)
            }
            FileType::AdditionalResource => {
                let additional_resource_id = self
                    .additional_resource_id
                    .ok_or(ThothError::AdditionalResourceFileUploadMissingAdditionalResourceId)?;
                File::from_additional_resource_id(db, &additional_resource_id)
            }
            FileType::WorkFeaturedVideo => {
                let work_featured_video_id = self
                    .work_featured_video_id
                    .ok_or(ThothError::WorkFeaturedVideoFileUploadMissingWorkFeaturedVideoId)?;
                File::from_work_featured_video_id(db, &work_featured_video_id)
            }
        }
    }

    pub(crate) fn persist_file_record<C: PolicyContext>(
        &self,
        ctx: &C,
        canonical_key: &str,
        cdn_url: &str,
        mime_type: &str,
        bytes: i64,
    ) -> ThothResult<(File, Option<String>)> {
        use crate::schema::file::dsl as file_dsl;

        let existing_file = self.existing_file(ctx.db())?;
        let old_object_key = existing_file.as_ref().map(|file| file.object_key.clone());

        let file = if let Some(existing) = existing_file {
            let mut connection = ctx.db().get()?;
            diesel::update(file_dsl::file.find(existing.file_id))
                .set((
                    file_dsl::object_key.eq(canonical_key),
                    file_dsl::cdn_url.eq(cdn_url),
                    file_dsl::mime_type.eq(mime_type),
                    file_dsl::bytes.eq(bytes),
                    file_dsl::sha256.eq(&self.declared_sha256),
                ))
                .get_result::<File>(&mut connection)
                .map_err(ThothError::from)?
        } else {
            let new_file = NewFile {
                file_type: self.file_type,
                work_id: self.work_id,
                publication_id: self.publication_id,
                additional_resource_id: self.additional_resource_id,
                work_featured_video_id: self.work_featured_video_id,
                object_key: canonical_key.to_string(),
                cdn_url: cdn_url.to_string(),
                mime_type: mime_type.to_string(),
                bytes,
                sha256: self.declared_sha256.clone(),
            };
            FilePolicy::can_create(ctx, &new_file, ())?;
            File::create(ctx.db(), &new_file)?
        };

        Ok((file, old_object_key))
    }

    pub(crate) fn sync_related_metadata<C: PolicyContext>(
        &self,
        ctx: &C,
        work: &Work,
        cdn_url: &str,
        featured_video_dimensions: Option<(i32, i32)>,
    ) -> ThothResult<()> {
        match self.file_type {
            FileType::Frontcover => {
                let mut patch: PatchWork = work.clone().into();
                patch.cover_url = Some(cdn_url.to_string());
                work.update(ctx, &patch)?;
            }
            FileType::Publication => {
                let publication_id = self
                    .publication_id
                    .ok_or(ThothError::PublicationFileUploadMissingPublicationId)?;
                Self::upsert_thoth_location(
                    ctx,
                    publication_id,
                    work.landing_page.clone(),
                    cdn_url,
                )?;
            }
            FileType::AdditionalResource => {
                let additional_resource_id = self
                    .additional_resource_id
                    .ok_or(ThothError::AdditionalResourceFileUploadMissingAdditionalResourceId)?;
                let additional_resource: AdditionalResource =
                    ctx.load_current(&additional_resource_id)?;
                let patch = PatchAdditionalResource {
                    additional_resource_id: additional_resource.additional_resource_id,
                    work_id: additional_resource.work_id,
                    title: additional_resource.title.clone(),
                    description: additional_resource.description.clone(),
                    attribution: additional_resource.attribution.clone(),
                    resource_type: additional_resource.resource_type,
                    doi: additional_resource.doi.clone(),
                    handle: additional_resource.handle.clone(),
                    url: Some(cdn_url.to_string()),
                    resource_ordinal: additional_resource.resource_ordinal,
                };
                additional_resource.update(ctx, &patch)?;
            }
            FileType::WorkFeaturedVideo => {
                let work_featured_video_id = self
                    .work_featured_video_id
                    .ok_or(ThothError::WorkFeaturedVideoFileUploadMissingWorkFeaturedVideoId)?;
                let work_featured_video: WorkFeaturedVideo =
                    ctx.load_current(&work_featured_video_id)?;
                let (width, height) = featured_video_dimensions
                    .unwrap_or((work_featured_video.width, work_featured_video.height));
                let patch = PatchWorkFeaturedVideo {
                    work_featured_video_id: work_featured_video.work_featured_video_id,
                    work_id: work_featured_video.work_id,
                    title: work_featured_video.title.clone(),
                    url: Some(cdn_url.to_string()),
                    width,
                    height,
                };
                work_featured_video.update(ctx, &patch)?;
            }
        }

        Ok(())
    }

    fn upsert_thoth_location<C: PolicyContext>(
        ctx: &C,
        publication_id: Uuid,
        landing_page: Option<String>,
        full_text_url: &str,
    ) -> ThothResult<()> {
        use crate::schema::location::dsl;

        let mut connection = ctx.db().get()?;

        let thoth_location = dsl::location
            .filter(dsl::publication_id.eq(publication_id))
            .filter(dsl::location_platform.eq(LocationPlatform::Thoth))
            .first::<Location>(&mut connection)
            .optional()
            .map_err(ThothError::from)?;

        if let Some(location) = thoth_location {
            let mut patch = PatchLocation::from(location.clone());
            patch.full_text_url = Some(full_text_url.to_string());
            patch.landing_page = landing_page;
            patch.canonical = true;
            if patch.canonical {
                patch.canonical_record_complete(ctx.db())?;
            }
            location.update(ctx, &patch)?;
            return Ok(());
        }

        let existing_canonical = dsl::location
            .filter(dsl::publication_id.eq(publication_id))
            .filter(dsl::canonical.eq(true))
            .first::<Location>(&mut connection)
            .optional()
            .map_err(ThothError::from)?;

        if existing_canonical.is_some() {
            let new_location = NewLocation {
                publication_id,
                landing_page,
                full_text_url: Some(full_text_url.to_string()),
                location_platform: LocationPlatform::Thoth,
                canonical: false,
            };
            let created_location = Location::create(ctx.db(), &new_location)?;
            let mut patch = PatchLocation::from(created_location.clone());
            patch.canonical = true;
            if patch.canonical {
                patch.canonical_record_complete(ctx.db())?;
            }
            created_location.update(ctx, &patch)?;
        } else {
            let new_location = NewLocation {
                publication_id,
                landing_page,
                full_text_url: Some(full_text_url.to_string()),
                location_platform: LocationPlatform::Thoth,
                canonical: true,
            };
            new_location.canonical_record_complete(ctx.db())?;
            Location::create(ctx.db(), &new_location)?;
        }

        Ok(())
    }
}
