use super::FileType;
use super::{File, FileUpload, NewFile, NewFileUpload};
use crate::model::Crud;
use crate::schema::{file, file_upload};
use diesel::prelude::*;
use diesel::OptionalExtension;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

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
        _db: &crate::db::PgPool,
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
        _db: &crate::db::PgPool,
        _filter: Option<String>,
        _publishers: Vec<Uuid>,
        _filter_param_1: Vec<Self::FilterParameter1>,
        _filter_param_2: Vec<Self::FilterParameter2>,
        _filter_param_3: Option<Self::FilterParameter3>,
        _filter_param_4: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        unimplemented!()
    }

    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        let mut connection = db.get()?;
        file::table
            .find(entity_id)
            .get_result::<File>(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    fn create(db: &crate::db::PgPool, data: &NewFile) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::insert_into(file::table)
            .values(data)
            .get_result::<File>(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    fn update(
        &self,
        _db: &crate::db::PgPool,
        _data: &NewFile,
        _account_id: &Uuid,
    ) -> ThothResult<Self> {
        unimplemented!()
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        let mut connection = db.get()?;
        diesel::delete(file::table.find(self.file_id))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        match (self.work_id, self.publication_id) {
            (Some(work_id), None) => {
                crate::model::work::Work::from_id(db, &work_id)?.publisher_id(db)
            }
            (None, Some(publication_id)) => {
                crate::model::publication::Publication::from_id(db, &publication_id)?
                    .publisher_id(db)
            }
            _ => Err(ThothError::FileMissingWorkOrPublicationId),
        }
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
        _db: &crate::db::PgPool,
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
        _db: &crate::db::PgPool,
        _filter: Option<String>,
        _publishers: Vec<Uuid>,
        _filter_param_1: Vec<Self::FilterParameter1>,
        _filter_param_2: Vec<Self::FilterParameter2>,
        _filter_param_3: Option<Self::FilterParameter3>,
        _filter_param_4: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        unimplemented!()
    }

    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        let mut connection = db.get()?;
        file_upload::table
            .find(entity_id)
            .get_result::<FileUpload>(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    fn create(db: &crate::db::PgPool, data: &NewFileUpload) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::insert_into(file_upload::table)
            .values(data)
            .get_result::<FileUpload>(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    fn update(
        &self,
        _db: &crate::db::PgPool,
        _data: &NewFileUpload,
        _account_id: &Uuid,
    ) -> ThothResult<Self> {
        unimplemented!()
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        let mut connection = db.get()?;
        diesel::delete(file_upload::table.find(self.file_upload_id))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        match (self.work_id, self.publication_id) {
            (Some(work_id), None) => {
                crate::model::work::Work::from_id(db, &work_id)?.publisher_id(db)
            }
            (None, Some(publication_id)) => {
                crate::model::publication::Publication::from_id(db, &publication_id)?
                    .publisher_id(db)
            }
            _ => Err(ThothError::FileUploadMissingWorkOrPublicationId),
        }
    }
}

impl File {
    pub fn from_object_key(db: &crate::db::PgPool, object_key: &str) -> ThothResult<Self> {
        use crate::schema::file::dsl;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::object_key.eq(object_key))
            .first::<File>(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    pub fn from_work_id(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<Option<Self>> {
        use crate::schema::file::dsl;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::work_id.eq(work_id))
            .filter(dsl::file_type.eq(FileType::Frontcover))
            .first::<File>(&mut connection)
            .optional()
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }

    pub fn from_publication_id(
        db: &crate::db::PgPool,
        publication_id: &Uuid,
    ) -> ThothResult<Option<Self>> {
        use crate::schema::file::dsl;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let mut connection = db.get()?;
        dsl::file
            .filter(dsl::publication_id.eq(publication_id))
            .filter(dsl::file_type.eq(FileType::Publication))
            .first::<File>(&mut connection)
            .optional()
            .map_err(|e: diesel::result::Error| ThothError::from(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_pool, PgPool};
    use crate::model::{Publication, Timestamp, Work};
    use dotenv::dotenv;
    use std::env;

    fn get_pool() -> PgPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .expect("TEST_DATABASE_URL or DATABASE_URL must be set");
        init_pool(&database_url)
    }

    fn create_test_work(db: &PgPool) -> Work {
        use crate::model::imprint::{Imprint, NewImprint};
        use crate::model::publisher::Publisher;
        use crate::model::work::{NewWork, WorkStatus, WorkType};

        let publisher = Publisher::create(
            db,
            &crate::model::publisher::NewPublisher {
                publisher_name: format!("Test Publisher {}", Uuid::new_v4()),
                publisher_shortname: None,
                publisher_url: None,
                accessibility_statement: None,
                accessibility_report_url: None,
            },
        )
        .expect("Failed to create test publisher");

        let imprint = Imprint::create(
            db,
            &NewImprint {
                publisher_id: publisher.publisher_id,
                imprint_name: format!("Test Imprint {}", Uuid::new_v4()),
                imprint_url: None,
                crossmark_doi: None,
            },
        )
        .expect("Failed to create test imprint");

        Work::create(
            db,
            &NewWork {
                work_type: WorkType::Monograph,
                work_status: WorkStatus::Active,
                reference: None,
                edition: None,
                imprint_id: imprint.imprint_id,
                doi: None,
                publication_date: None,
                withdrawn_date: None,
                place: None,
                page_count: None,
                page_breakdown: None,
                image_count: None,
                table_count: None,
                audio_count: None,
                video_count: None,
                license: None,
                copyright_holder: None,
                landing_page: None,
                lccn: None,
                oclc: None,
                general_note: None,
                bibliography_note: None,
                toc: None,
                cover_url: None,
                cover_caption: None,
                first_page: None,
                last_page: None,
                page_interval: None,
            },
        )
        .expect("Failed to create test work")
    }

    fn create_test_publication(db: &PgPool, work_id: Uuid) -> Publication {
        use crate::model::publication::{NewPublication, PublicationType};

        Publication::create(
            db,
            &NewPublication {
                publication_type: PublicationType::Pdf,
                work_id,
                isbn: None,
                width_mm: None,
                width_in: None,
                height_mm: None,
                height_in: None,
                depth_mm: None,
                depth_in: None,
                weight_g: None,
                weight_oz: None,
                accessibility_standard: None,
                accessibility_additional_standard: None,
                accessibility_exception: None,
                accessibility_report_url: None,
            },
        )
        .expect("Failed to create test publication")
    }

    #[test]
    fn test_file_from_id() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_file = NewFile {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            object_key: format!("test/{}", Uuid::new_v4()),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: "abc123".repeat(8), // 48 chars
        };

        let created_file = File::create(&db, &new_file).expect("Failed to create file");
        let retrieved_file =
            File::from_id(&db, &created_file.file_id).expect("Failed to retrieve file");

        assert_eq!(created_file.file_id, retrieved_file.file_id);
        assert_eq!(created_file.object_key, retrieved_file.object_key);
        assert_eq!(created_file.file_type, retrieved_file.file_type);
    }

    #[test]
    fn test_file_create_and_delete() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_file = NewFile {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            object_key: format!("test/{}", Uuid::new_v4()),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: "abc123".repeat(8),
        };

        let created_file = File::create(&db, &new_file).expect("Failed to create file");
        let file_id = created_file.file_id;

        let retrieved = File::from_id(&db, &file_id).expect("File should exist");
        assert_eq!(retrieved.file_id, file_id);

        let deleted_file = created_file.delete(&db).expect("Failed to delete file");
        assert_eq!(deleted_file.file_id, file_id);

        let result = File::from_id(&db, &file_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_from_object_key() {
        let db = get_pool();
        let work = create_test_work(&db);

        let object_key = format!("test/{}", Uuid::new_v4());
        let new_file = NewFile {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            object_key: object_key.clone(),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: "abc123".repeat(8),
        };

        let created_file = File::create(&db, &new_file).expect("Failed to create file");

        let retrieved_file =
            File::from_object_key(&db, &object_key).expect("Failed to retrieve file by object_key");
        assert_eq!(created_file.file_id, retrieved_file.file_id);
        assert_eq!(object_key, retrieved_file.object_key);
    }

    #[test]
    fn test_file_from_work_id() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_file = NewFile {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            object_key: format!("test/{}", Uuid::new_v4()),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: "abc123".repeat(8),
        };

        let _created_file = File::create(&db, &new_file).expect("Failed to create file");

        let retrieved_file = File::from_work_id(&db, &work.work_id)
            .expect("Failed to retrieve file by work_id")
            .expect("File should exist");

        assert_eq!(retrieved_file.work_id, Some(work.work_id));
        assert_eq!(retrieved_file.file_type, FileType::Frontcover);
    }

    #[test]
    fn test_file_from_work_id_not_found() {
        let db = get_pool();
        let non_existent_work_id = Uuid::new_v4();

        let result = File::from_work_id(&db, &non_existent_work_id).expect("Query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_file_from_publication_id() {
        let db = get_pool();
        let work = create_test_work(&db);
        let publication = create_test_publication(&db, work.work_id);

        let new_file = NewFile {
            file_type: FileType::Publication,
            work_id: None,
            publication_id: Some(publication.publication_id),
            object_key: format!("test/{}", Uuid::new_v4()),
            cdn_url: "https://cdn.example.com/test.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            bytes: 2048,
            sha256: "def456".repeat(8),
        };

        let _created_file = File::create(&db, &new_file).expect("Failed to create file");

        let retrieved_file = File::from_publication_id(&db, &publication.publication_id)
            .expect("Failed to retrieve file by publication_id")
            .expect("File should exist");

        assert_eq!(
            retrieved_file.publication_id,
            Some(publication.publication_id)
        );
        assert_eq!(retrieved_file.file_type, FileType::Publication);
    }

    #[test]
    fn test_file_from_publication_id_not_found() {
        let db = get_pool();
        let non_existent_publication_id = Uuid::new_v4();

        let result = File::from_publication_id(&db, &non_existent_publication_id)
            .expect("Query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_file_publisher_id_from_work() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_file = NewFile {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            object_key: format!("test/{}", Uuid::new_v4()),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: "abc123".repeat(8),
        };

        let created_file = File::create(&db, &new_file).expect("Failed to create file");
        let publisher_id = created_file
            .publisher_id(&db)
            .expect("Failed to get publisher_id");

        assert!(!publisher_id.is_nil());
    }

    #[test]
    fn test_file_publisher_id_from_publication() {
        let db = get_pool();
        let work = create_test_work(&db);
        let publication = create_test_publication(&db, work.work_id);

        let new_file = NewFile {
            file_type: FileType::Publication,
            work_id: None,
            publication_id: Some(publication.publication_id),
            object_key: format!("test/{}", Uuid::new_v4()),
            cdn_url: "https://cdn.example.com/test.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            bytes: 2048,
            sha256: "def456".repeat(8),
        };

        let created_file = File::create(&db, &new_file).expect("Failed to create file");
        let publisher_id = created_file
            .publisher_id(&db)
            .expect("Failed to get publisher_id");

        assert!(!publisher_id.is_nil());
    }

    #[test]
    fn test_file_publisher_id_error_when_missing_both() {
        let db = get_pool();

        let file = File {
            file_id: Uuid::new_v4(),
            file_type: FileType::Frontcover,
            work_id: None,
            publication_id: None,
            object_key: "test/key".to_string(),
            cdn_url: "https://cdn.example.com/test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            bytes: 1024,
            sha256: "abc123".repeat(8),
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        };

        let result = file.publisher_id(&db);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ThothError::FileMissingWorkOrPublicationId)
        ));
    }

    #[test]
    fn test_file_upload_from_id() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_upload = NewFileUpload {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: "abc123".repeat(8),
        };

        let created_upload =
            FileUpload::create(&db, &new_upload).expect("Failed to create file upload");
        let retrieved_upload = FileUpload::from_id(&db, &created_upload.file_upload_id)
            .expect("Failed to retrieve file upload");

        assert_eq!(
            created_upload.file_upload_id,
            retrieved_upload.file_upload_id
        );
        assert_eq!(
            created_upload.declared_extension,
            retrieved_upload.declared_extension
        );
    }

    #[test]
    fn test_file_upload_create_and_delete() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_upload = NewFileUpload {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: "abc123".repeat(8),
        };

        let created_upload =
            FileUpload::create(&db, &new_upload).expect("Failed to create file upload");
        let upload_id = created_upload.file_upload_id;

        let retrieved = FileUpload::from_id(&db, &upload_id).expect("FileUpload should exist");
        assert_eq!(retrieved.file_upload_id, upload_id);

        let deleted_upload = created_upload
            .delete(&db)
            .expect("Failed to delete file upload");
        assert_eq!(deleted_upload.file_upload_id, upload_id);

        let result = FileUpload::from_id(&db, &upload_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_upload_publisher_id_from_work() {
        let db = get_pool();
        let work = create_test_work(&db);

        let new_upload = NewFileUpload {
            file_type: FileType::Frontcover,
            work_id: Some(work.work_id),
            publication_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: "abc123".repeat(8),
        };

        let created_upload =
            FileUpload::create(&db, &new_upload).expect("Failed to create file upload");
        let publisher_id = created_upload
            .publisher_id(&db)
            .expect("Failed to get publisher_id");

        assert!(!publisher_id.is_nil());
    }

    #[test]
    fn test_file_upload_publisher_id_from_publication() {
        let db = get_pool();
        let work = create_test_work(&db);
        let publication = create_test_publication(&db, work.work_id);

        let new_upload = NewFileUpload {
            file_type: FileType::Publication,
            work_id: None,
            publication_id: Some(publication.publication_id),
            declared_mime_type: "application/pdf".to_string(),
            declared_extension: "pdf".to_string(),
            declared_sha256: "def456".repeat(8),
        };

        let created_upload =
            FileUpload::create(&db, &new_upload).expect("Failed to create file upload");
        let publisher_id = created_upload
            .publisher_id(&db)
            .expect("Failed to get publisher_id");

        assert!(!publisher_id.is_nil());
    }

    #[test]
    fn test_file_upload_publisher_id_error_when_missing_both() {
        let db = get_pool();

        let upload = FileUpload {
            file_upload_id: Uuid::new_v4(),
            file_type: FileType::Frontcover,
            work_id: None,
            publication_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: "abc123".repeat(8),
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        };

        let result = upload.publisher_id(&db);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ThothError::FileUploadMissingWorkOrPublicationId)
        ));
    }
}
