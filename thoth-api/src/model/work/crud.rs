use super::{
    NewWork, NewWorkHistory, PatchWork, Work, WorkField, WorkHistory, WorkOrderBy, WorkStatus,
    WorkType,
};
use crate::graphql::utils::Direction;
use crate::model::{Convert, Crud, DbInsert, Doi, HistoryEntry, LengthUnit};
use crate::schema::{work, work_history};
use crate::{crud_methods, db_insert};
use diesel::dsl::any;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Work {
    pub fn from_doi(
        db: &crate::db::PgPool,
        doi: Doi,
        work_types: Vec<WorkType>,
    ) -> ThothResult<Self> {
        use crate::schema::work::dsl;
        use diesel::sql_types::Nullable;
        use diesel::sql_types::Text;
        let connection = db.get().unwrap();
        // Allow case-insensitive searching (DOIs in database may have mixed casing)
        sql_function!(fn lower(x: Nullable<Text>) -> Nullable<Text>);
        let mut query = dsl::work
            .filter(lower(dsl::doi).eq(doi.to_lowercase_string()))
            .into_boxed();
        if !work_types.is_empty() {
            query = query.filter(dsl::work_type.eq(any(work_types)));
        }
        query.get_result::<Work>(&connection).map_err(|e| e.into())
    }

    pub fn can_update_imprint(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        use crate::schema::issue::dsl::*;
        let connection = db.get().unwrap();
        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        let issue_count = issue
            .filter(work_id.eq(self.work_id))
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading issue count for work")
            .to_string()
            .parse::<i32>()
            .unwrap();
        // If a work has any related issues, its imprint cannot be changed,
        // because an issue's series and work must both have the same imprint.
        if issue_count == 0 {
            Ok(())
        } else {
            Err(ThothError::IssueImprintsError)
        }
    }

    pub fn can_be_chapter(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        use crate::schema::publication::dsl::*;
        let connection = db.get().unwrap();
        let isbn_count = publication
            .filter(work_id.eq(self.work_id))
            .filter(isbn.is_not_null())
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading publication ISBNs for work")
            .to_string()
            .parse::<i32>()
            .unwrap();
        // If a work has any publications with ISBNs,
        // its type cannot be changed to Book Chapter.
        if isbn_count == 0 {
            Ok(())
        } else {
            Err(ThothError::ChapterIsbnError)
        }
    }

    pub fn update_with_units(
        &self,
        db: &crate::db::PgPool,
        data: PatchWork,
        account_id: &Uuid,
        units: LengthUnit,
    ) -> ThothResult<Self> {
        if units == LengthUnit::Mm {
            // Data is already in units compatible with the database -
            // no conversions required before/after updating
            self.update(db, &data, account_id)
        } else {
            let mut converted_data = data;
            converted_data.width = converted_data
                .width
                .map(|w| w.convert_units_from_to(&units, &LengthUnit::Mm));
            converted_data.height = converted_data
                .height
                .map(|h| h.convert_units_from_to(&units, &LengthUnit::Mm));
            let result = self.update(db, &converted_data, account_id);
            if let Ok(mut retrieved_data) = result {
                retrieved_data.width = retrieved_data
                    .width
                    .map(|w| w.convert_units_from_to(&LengthUnit::Mm, &units));
                retrieved_data.height = retrieved_data
                    .height
                    .map(|h| h.convert_units_from_to(&LengthUnit::Mm, &units));
                Ok(retrieved_data)
            } else {
                result
            }
        }
    }

    pub fn create_with_units(
        db: &crate::db::PgPool,
        data: NewWork,
        units: LengthUnit,
    ) -> ThothResult<Self> {
        if units == LengthUnit::Mm {
            // Data is already in units compatible with the database -
            // no conversions required before/after creating
            Self::create(db, &data)
        } else {
            let mut converted_data = data;
            converted_data.width = converted_data
                .width
                .map(|w| w.convert_units_from_to(&units, &LengthUnit::Mm));
            converted_data.height = converted_data
                .height
                .map(|h| h.convert_units_from_to(&units, &LengthUnit::Mm));
            let result = Self::create(db, &converted_data);
            if let Ok(mut retrieved_data) = result {
                retrieved_data.width = retrieved_data
                    .width
                    .map(|w| w.convert_units_from_to(&LengthUnit::Mm, &units));
                retrieved_data.height = retrieved_data
                    .height
                    .map(|h| h.convert_units_from_to(&LengthUnit::Mm, &units));
                Ok(retrieved_data)
            } else {
                result
            }
        }
    }
}

impl Crud for Work {
    type NewEntity = NewWork;
    type PatchEntity = PatchWork;
    type OrderByEntity = WorkOrderBy;
    type FilterParameter1 = WorkType;
    type FilterParameter2 = WorkStatus;

    fn pk(&self) -> Uuid {
        self.work_id
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
        work_types: Vec<Self::FilterParameter1>,
        work_status: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Work>> {
        use crate::schema::work::dsl;
        let connection = db.get().unwrap();
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .select((
                dsl::work_id,
                dsl::work_type,
                dsl::work_status,
                dsl::full_title,
                dsl::title,
                dsl::subtitle,
                dsl::reference,
                dsl::edition,
                dsl::imprint_id,
                dsl::doi,
                dsl::publication_date,
                dsl::place,
                dsl::width,
                dsl::height,
                dsl::page_count,
                dsl::page_breakdown,
                dsl::image_count,
                dsl::table_count,
                dsl::audio_count,
                dsl::video_count,
                dsl::license,
                dsl::copyright_holder,
                dsl::landing_page,
                dsl::lccn,
                dsl::oclc,
                dsl::short_abstract,
                dsl::long_abstract,
                dsl::general_note,
                dsl::toc,
                dsl::cover_url,
                dsl::cover_caption,
                dsl::created_at,
                dsl::updated_at,
                dsl::first_page,
                dsl::last_page,
                dsl::page_interval,
            ))
            .into_boxed();

        match order.field {
            WorkField::WorkId => match order.direction {
                Direction::Asc => query = query.order(dsl::work_id.asc()),
                Direction::Desc => query = query.order(dsl::work_id.desc()),
            },
            WorkField::WorkType => match order.direction {
                Direction::Asc => query = query.order(dsl::work_type.asc()),
                Direction::Desc => query = query.order(dsl::work_type.desc()),
            },
            WorkField::WorkStatus => match order.direction {
                Direction::Asc => query = query.order(dsl::work_status.asc()),
                Direction::Desc => query = query.order(dsl::work_status.desc()),
            },
            WorkField::FullTitle => match order.direction {
                Direction::Asc => query = query.order(dsl::full_title.asc()),
                Direction::Desc => query = query.order(dsl::full_title.desc()),
            },
            WorkField::Title => match order.direction {
                Direction::Asc => query = query.order(dsl::title.asc()),
                Direction::Desc => query = query.order(dsl::title.desc()),
            },
            WorkField::Subtitle => match order.direction {
                Direction::Asc => query = query.order(dsl::subtitle.asc()),
                Direction::Desc => query = query.order(dsl::subtitle.desc()),
            },
            WorkField::Reference => match order.direction {
                Direction::Asc => query = query.order(dsl::reference.asc()),
                Direction::Desc => query = query.order(dsl::reference.desc()),
            },
            WorkField::Edition => match order.direction {
                Direction::Asc => query = query.order(dsl::edition.asc()),
                Direction::Desc => query = query.order(dsl::edition.desc()),
            },
            WorkField::Doi => match order.direction {
                Direction::Asc => query = query.order(dsl::doi.asc()),
                Direction::Desc => query = query.order(dsl::doi.desc()),
            },
            WorkField::PublicationDate => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_date.asc()),
                Direction::Desc => query = query.order(dsl::publication_date.desc()),
            },
            WorkField::Place => match order.direction {
                Direction::Asc => query = query.order(dsl::place.asc()),
                Direction::Desc => query = query.order(dsl::place.desc()),
            },
            WorkField::Width => match order.direction {
                Direction::Asc => query = query.order(dsl::width.asc()),
                Direction::Desc => query = query.order(dsl::width.desc()),
            },
            WorkField::Height => match order.direction {
                Direction::Asc => query = query.order(dsl::height.asc()),
                Direction::Desc => query = query.order(dsl::height.desc()),
            },
            WorkField::PageCount => match order.direction {
                Direction::Asc => query = query.order(dsl::page_count.asc()),
                Direction::Desc => query = query.order(dsl::page_count.desc()),
            },
            WorkField::PageBreakdown => match order.direction {
                Direction::Asc => query = query.order(dsl::page_breakdown.asc()),
                Direction::Desc => query = query.order(dsl::page_breakdown.desc()),
            },
            WorkField::FirstPage => match order.direction {
                Direction::Asc => query = query.order(dsl::first_page.asc()),
                Direction::Desc => query = query.order(dsl::first_page.desc()),
            },
            WorkField::LastPage => match order.direction {
                Direction::Asc => query = query.order(dsl::last_page.asc()),
                Direction::Desc => query = query.order(dsl::last_page.desc()),
            },
            WorkField::PageInterval => match order.direction {
                Direction::Asc => query = query.order(dsl::page_breakdown.asc()),
                Direction::Desc => query = query.order(dsl::page_breakdown.desc()),
            },
            WorkField::ImageCount => match order.direction {
                Direction::Asc => query = query.order(dsl::image_count.asc()),
                Direction::Desc => query = query.order(dsl::image_count.desc()),
            },
            WorkField::TableCount => match order.direction {
                Direction::Asc => query = query.order(dsl::table_count.asc()),
                Direction::Desc => query = query.order(dsl::table_count.desc()),
            },
            WorkField::AudioCount => match order.direction {
                Direction::Asc => query = query.order(dsl::audio_count.asc()),
                Direction::Desc => query = query.order(dsl::audio_count.desc()),
            },
            WorkField::VideoCount => match order.direction {
                Direction::Asc => query = query.order(dsl::video_count.asc()),
                Direction::Desc => query = query.order(dsl::video_count.desc()),
            },
            WorkField::License => match order.direction {
                Direction::Asc => query = query.order(dsl::license.asc()),
                Direction::Desc => query = query.order(dsl::license.desc()),
            },
            WorkField::CopyrightHolder => match order.direction {
                Direction::Asc => query = query.order(dsl::copyright_holder.asc()),
                Direction::Desc => query = query.order(dsl::copyright_holder.desc()),
            },
            WorkField::LandingPage => match order.direction {
                Direction::Asc => query = query.order(dsl::landing_page.asc()),
                Direction::Desc => query = query.order(dsl::landing_page.desc()),
            },
            WorkField::Lccn => match order.direction {
                Direction::Asc => query = query.order(dsl::lccn.asc()),
                Direction::Desc => query = query.order(dsl::lccn.desc()),
            },
            WorkField::Oclc => match order.direction {
                Direction::Asc => query = query.order(dsl::oclc.asc()),
                Direction::Desc => query = query.order(dsl::oclc.desc()),
            },
            WorkField::ShortAbstract => match order.direction {
                Direction::Asc => query = query.order(dsl::short_abstract.asc()),
                Direction::Desc => query = query.order(dsl::short_abstract.desc()),
            },
            WorkField::LongAbstract => match order.direction {
                Direction::Asc => query = query.order(dsl::long_abstract.asc()),
                Direction::Desc => query = query.order(dsl::long_abstract.desc()),
            },
            WorkField::GeneralNote => match order.direction {
                Direction::Asc => query = query.order(dsl::general_note.asc()),
                Direction::Desc => query = query.order(dsl::general_note.desc()),
            },
            WorkField::Toc => match order.direction {
                Direction::Asc => query = query.order(dsl::toc.asc()),
                Direction::Desc => query = query.order(dsl::toc.desc()),
            },
            WorkField::CoverUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::cover_url.asc()),
                Direction::Desc => query = query.order(dsl::cover_url.desc()),
            },
            WorkField::CoverCaption => match order.direction {
                Direction::Asc => query = query.order(dsl::cover_caption.asc()),
                Direction::Desc => query = query.order(dsl::cover_caption.desc()),
            },
            WorkField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            WorkField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq(any(publishers)));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::imprint_id.eq(pid));
        }
        if !work_types.is_empty() {
            query = query.filter(dsl::work_type.eq(any(work_types)));
        }
        if let Some(wk_status) = work_status {
            query = query.filter(dsl::work_status.eq(wk_status));
        }
        if let Some(filter) = filter {
            query = query.filter(
                dsl::full_title
                    .ilike(format!("%{}%", filter))
                    .or(dsl::doi.ilike(format!("%{}%", filter)))
                    .or(dsl::reference.ilike(format!("%{}%", filter)))
                    .or(dsl::short_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::long_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::landing_page.ilike(format!("%{}%", filter))),
            );
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Work>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        work_types: Vec<Self::FilterParameter1>,
        work_status: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::work::dsl;
        let connection = db.get().unwrap();
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq(any(publishers)));
        }
        if !work_types.is_empty() {
            query = query.filter(dsl::work_type.eq(any(work_types)));
        }
        if let Some(wk_status) = work_status {
            query = query.filter(dsl::work_status.eq(wk_status));
        }
        if let Some(filter) = filter {
            query = query.filter(
                dsl::full_title
                    .ilike(format!("%{}%", filter))
                    .or(dsl::doi.ilike(format!("%{}%", filter)))
                    .or(dsl::reference.ilike(format!("%{}%", filter)))
                    .or(dsl::short_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::long_abstract.ilike(format!("%{}%", filter)))
                    .or(dsl::landing_page.ilike(format!("%{}%", filter))),
            );
        }

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        match query.count().get_result::<i64>(&connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::imprint::Imprint::from_id(db, &self.imprint_id)?.publisher_id(db)
    }

    crud_methods!(work::table, work::dsl::work);
}

impl HistoryEntry for Work {
    type NewHistoryEntity = NewWorkHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            work_id: self.work_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewWorkHistory {
    type MainEntity = WorkHistory;

    db_insert!(work_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_pk() {
        let work: Work = Default::default();
        assert_eq!(work.pk(), work.work_id);
    }

    #[test]
    fn test_new_work_history_from_work() {
        let work: Work = Default::default();
        let account_id: Uuid = Default::default();
        let new_work_history = work.new_history_entry(&account_id);
        assert_eq!(new_work_history.work_id, work.work_id);
        assert_eq!(new_work_history.account_id, account_id);
        assert_eq!(
            new_work_history.data,
            serde_json::Value::String(serde_json::to_string(&work).unwrap())
        );
    }
}
