use super::{
    NewWorkFeaturedVideo, NewWorkFeaturedVideoHistory, PatchWorkFeaturedVideo, WorkFeaturedVideo,
};
use crate::model::work::WorkType;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{work, work_featured_video, work_featured_video_history};
use diesel::{Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

fn validate_book_only(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<()> {
    let mut connection = db.get()?;
    let work_type = work::table
        .select(work::work_type)
        .filter(work::work_id.eq(work_id))
        .first::<WorkType>(&mut connection)?;

    if work_type == WorkType::BookChapter {
        return Err(ThothError::InternalError(
            "Featured videos can only be attached to book records, not chapters".to_string(),
        ));
    }
    Ok(())
}

impl Crud for WorkFeaturedVideo {
    type NewEntity = NewWorkFeaturedVideo;
    type PatchEntity = PatchWorkFeaturedVideo;
    type OrderByEntity = ();
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.work_featured_video_id
    }

    fn all(
        _db: &crate::db::PgPool,
        _limit: i32,
        _offset: i32,
        _filter: Option<String>,
        _order: Self::OrderByEntity,
        _publishers: Vec<Uuid>,
        _parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<WorkFeaturedVideo>> {
        Err(ThothError::InternalError(
            "WorkFeaturedVideo does not support list queries".to_string(),
        ))
    }

    fn count(
        _db: &crate::db::PgPool,
        _filter: Option<String>,
        _publishers: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        Err(ThothError::InternalError(
            "WorkFeaturedVideo does not support count queries".to_string(),
        ))
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> ThothResult<Self> {
        validate_book_only(db, &data.work_id)?;
        let mut connection = db.get()?;
        diesel::insert_into(work_featured_video::table)
            .values(data)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn update(
        &self,
        db: &crate::db::PgPool,
        data: &Self::PatchEntity,
        account_id: &Uuid,
    ) -> ThothResult<Self> {
        validate_book_only(db, &self.work_id)?;

        let mut connection = db.get()?;
        connection.transaction(|connection| {
            diesel::update(work_featured_video::dsl::work_featured_video.find(&self.pk()))
                .set(data)
                .get_result(connection)
                .map_err(Into::into)
                .and_then(|c| {
                    self.new_history_entry(account_id)
                        .insert(connection)
                        .map(|_| c)
                })
        })
    }

    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = db.get()?;
        work_featured_video::dsl::work_featured_video
            .find(entity_id)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = db.get()?;
        diesel::delete(work_featured_video::dsl::work_featured_video.find(&self.pk()))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(Into::into)
    }
}

impl WorkFeaturedVideo {
    pub fn from_work_id(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<Option<Self>> {
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        let mut connection = db.get()?;
        work_featured_video::table
            .filter(work_featured_video::work_id.eq(work_id))
            .first::<Self>(&mut connection)
            .optional()
            .map_err(Into::into)
    }
}

impl HistoryEntry for WorkFeaturedVideo {
    type NewHistoryEntity = NewWorkFeaturedVideoHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            work_featured_video_id: self.work_featured_video_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewWorkFeaturedVideoHistory {
    type MainEntity = super::WorkFeaturedVideoHistory;

    db_insert!(work_featured_video_history::table);
}
