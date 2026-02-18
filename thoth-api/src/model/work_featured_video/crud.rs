use super::{
    NewWorkFeaturedVideo, NewWorkFeaturedVideoHistory, PatchWorkFeaturedVideo, WorkFeaturedVideo,
    WorkFeaturedVideoField, WorkFeaturedVideoHistory, WorkFeaturedVideoOrderBy,
};
use crate::graphql::types::inputs::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{work_featured_video, work_featured_video_history};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for WorkFeaturedVideo {
    type NewEntity = NewWorkFeaturedVideo;
    type PatchEntity = PatchWorkFeaturedVideo;
    type OrderByEntity = WorkFeaturedVideoOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.work_featured_video_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<WorkFeaturedVideo>> {
        use crate::schema::work_featured_video::dsl::*;
        let mut connection = db.get()?;
        let mut query = work_featured_video
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::work_featured_video::all_columns)
            .into_boxed();

        query = match order.field {
            WorkFeaturedVideoField::WorkFeaturedVideoId => match order.direction {
                Direction::Asc => query.order(work_featured_video_id.asc()),
                Direction::Desc => query.order(work_featured_video_id.desc()),
            },
            WorkFeaturedVideoField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            WorkFeaturedVideoField::Title => match order.direction {
                Direction::Asc => query.order(title.asc()),
                Direction::Desc => query.order(title.desc()),
            },
            WorkFeaturedVideoField::Url => match order.direction {
                Direction::Asc => query.order(url.asc()),
                Direction::Desc => query.order(url.desc()),
            },
            WorkFeaturedVideoField::Width => match order.direction {
                Direction::Asc => query.order(width.asc()),
                Direction::Desc => query.order(width.desc()),
            },
            WorkFeaturedVideoField::Height => match order.direction {
                Direction::Asc => query.order(height.asc()),
                Direction::Desc => query.order(height.desc()),
            },
            WorkFeaturedVideoField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            WorkFeaturedVideoField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    title
                        .ilike(format!("%{filter}%"))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<WorkFeaturedVideo>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::work_featured_video::dsl::*;
        let mut connection = db.get()?;
        let mut query = work_featured_video
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    title
                        .ilike(format!("%{filter}%"))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }

        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(
        work_featured_video::table,
        work_featured_video::dsl::work_featured_video
    );
}

publisher_id_impls!(
    WorkFeaturedVideo,
    NewWorkFeaturedVideo,
    PatchWorkFeaturedVideo,
    |s, db| { crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db) }
);

impl WorkFeaturedVideo {
    pub fn from_work_id(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<Option<Self>> {
        use crate::schema::work_featured_video::dsl;

        let mut connection = db.get()?;
        dsl::work_featured_video
            .filter(dsl::work_id.eq(work_id))
            .first::<Self>(&mut connection)
            .optional()
            .map_err(Into::into)
    }
}

impl HistoryEntry for WorkFeaturedVideo {
    type NewHistoryEntity = NewWorkFeaturedVideoHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            work_featured_video_id: self.work_featured_video_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewWorkFeaturedVideoHistory {
    type MainEntity = WorkFeaturedVideoHistory;

    db_insert!(work_featured_video_history::table);
}
