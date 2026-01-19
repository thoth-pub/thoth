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
    use crate::model::Timestamp;

    #[test]
    fn test_file_pk() {
        let file_id = Uuid::new_v4();
        let file = File {
            file_id,
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

        assert_eq!(file.pk(), file_id);
    }

    #[test]
    fn test_file_upload_pk() {
        let file_upload_id = Uuid::new_v4();
        let upload = FileUpload {
            file_upload_id,
            file_type: FileType::Frontcover,
            work_id: None,
            publication_id: None,
            declared_mime_type: "image/jpeg".to_string(),
            declared_extension: "jpg".to_string(),
            declared_sha256: "abc123".repeat(8),
            created_at: Timestamp::default(),
            updated_at: Timestamp::default(),
        };

        assert_eq!(upload.pk(), file_upload_id);
    }
}
