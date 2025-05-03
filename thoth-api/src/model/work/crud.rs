use super::{
    NewWork, NewWorkHistory, PatchWork, Work, WorkField, WorkHistory, WorkOrderBy, WorkStatus,
    WorkType,
};
use crate::graphql::model::TimeExpression;
use crate::graphql::utils::{Direction, Expression};
use crate::model::work_relation::{RelationType, WorkRelation, WorkRelationOrderBy};
use crate::model::{Crud, DbInsert, Doi, HistoryEntry};
use crate::schema::{work, work_history};
use crate::{crud_methods, db_insert};
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
        let mut connection = db.get()?;
        // Allow case-insensitive searching (DOIs in database may have mixed casing)
        define_sql_function!(fn lower(x: Nullable<Text>) -> Nullable<Text>);
        let mut query = dsl::work
            .filter(lower(dsl::doi).eq(doi.to_lowercase_string()))
            .into_boxed();
        if !work_types.is_empty() {
            query = query.filter(dsl::work_type.eq_any(work_types));
        }
        query
            .get_result::<Work>(&mut connection)
            .map_err(|e| e.into())
    }

    pub fn can_update_imprint(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        use crate::schema::issue::dsl::*;
        let mut connection = db.get()?;
        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        let issue_count = issue
            .filter(work_id.eq(self.work_id))
            .count()
            .get_result::<i64>(&mut connection)
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
        let mut connection = db.get()?;
        let isbn_count = publication
            .filter(work_id.eq(self.work_id))
            .filter(isbn.is_not_null())
            .count()
            .get_result::<i64>(&mut connection)
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

    pub fn children(&self, db: &crate::db::PgPool) -> ThothResult<Vec<Work>> {
        WorkRelation::all(
            db,
            99999,
            0,
            None,
            WorkRelationOrderBy::default(),
            vec![],
            Some(self.work_id),
            None,
            vec![RelationType::HasChild],
            vec![],
            None,
        )
        .unwrap_or_default()
        .into_iter()
        .map(|relation| Work::from_id(db, &relation.related_work_id))
        .collect()
    }
}

impl Crud for Work {
    type NewEntity = NewWork;
    type PatchEntity = PatchWork;
    type OrderByEntity = WorkOrderBy;
    type FilterParameter1 = WorkType;
    type FilterParameter2 = WorkStatus;
    type FilterParameter3 = TimeExpression;

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
        work_statuses: Vec<Self::FilterParameter2>,
        updated_at_with_relations: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Work>> {
        use crate::schema::work::dsl;
        let mut connection = db.get()?;
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .select(crate::schema::work::all_columns)
            .into_boxed();

        query = match order.field {
            WorkField::WorkId => match order.direction {
                Direction::Asc => query.order(dsl::work_id.asc()),
                Direction::Desc => query.order(dsl::work_id.desc()),
            },
            WorkField::WorkType => match order.direction {
                Direction::Asc => query.order(dsl::work_type.asc()),
                Direction::Desc => query.order(dsl::work_type.desc()),
            },
            WorkField::WorkStatus => match order.direction {
                Direction::Asc => query.order(dsl::work_status.asc()),
                Direction::Desc => query.order(dsl::work_status.desc()),
            },
            WorkField::FullTitle => match order.direction {
                Direction::Asc => query.order(dsl::full_title.asc()),
                Direction::Desc => query.order(dsl::full_title.desc()),
            },
            WorkField::Title => match order.direction {
                Direction::Asc => query.order(dsl::title.asc()),
                Direction::Desc => query.order(dsl::title.desc()),
            },
            WorkField::Subtitle => match order.direction {
                Direction::Asc => query.order(dsl::subtitle.asc()),
                Direction::Desc => query.order(dsl::subtitle.desc()),
            },
            WorkField::Reference => match order.direction {
                Direction::Asc => query.order(dsl::reference.asc()),
                Direction::Desc => query.order(dsl::reference.desc()),
            },
            WorkField::Edition => match order.direction {
                Direction::Asc => query.order(dsl::edition.asc()),
                Direction::Desc => query.order(dsl::edition.desc()),
            },
            WorkField::Doi => match order.direction {
                Direction::Asc => query.order(dsl::doi.asc()),
                Direction::Desc => query.order(dsl::doi.desc()),
            },
            WorkField::PublicationDate => match order.direction {
                Direction::Asc => query.order(dsl::publication_date.asc()),
                Direction::Desc => query.order(dsl::publication_date.desc()),
            },
            WorkField::WithdrawnDate => match order.direction {
                Direction::Asc => query.order(dsl::withdrawn_date.asc()),
                Direction::Desc => query.order(dsl::withdrawn_date.desc()),
            },
            WorkField::Place => match order.direction {
                Direction::Asc => query.order(dsl::place.asc()),
                Direction::Desc => query.order(dsl::place.desc()),
            },
            WorkField::PageCount => match order.direction {
                Direction::Asc => query.order(dsl::page_count.asc()),
                Direction::Desc => query.order(dsl::page_count.desc()),
            },
            WorkField::PageBreakdown => match order.direction {
                Direction::Asc => query.order(dsl::page_breakdown.asc()),
                Direction::Desc => query.order(dsl::page_breakdown.desc()),
            },
            WorkField::FirstPage => match order.direction {
                Direction::Asc => query.order(dsl::first_page.asc()),
                Direction::Desc => query.order(dsl::first_page.desc()),
            },
            WorkField::LastPage => match order.direction {
                Direction::Asc => query.order(dsl::last_page.asc()),
                Direction::Desc => query.order(dsl::last_page.desc()),
            },
            WorkField::PageInterval => match order.direction {
                Direction::Asc => query.order(dsl::page_breakdown.asc()),
                Direction::Desc => query.order(dsl::page_breakdown.desc()),
            },
            WorkField::ImageCount => match order.direction {
                Direction::Asc => query.order(dsl::image_count.asc()),
                Direction::Desc => query.order(dsl::image_count.desc()),
            },
            WorkField::TableCount => match order.direction {
                Direction::Asc => query.order(dsl::table_count.asc()),
                Direction::Desc => query.order(dsl::table_count.desc()),
            },
            WorkField::AudioCount => match order.direction {
                Direction::Asc => query.order(dsl::audio_count.asc()),
                Direction::Desc => query.order(dsl::audio_count.desc()),
            },
            WorkField::VideoCount => match order.direction {
                Direction::Asc => query.order(dsl::video_count.asc()),
                Direction::Desc => query.order(dsl::video_count.desc()),
            },
            WorkField::License => match order.direction {
                Direction::Asc => query.order(dsl::license.asc()),
                Direction::Desc => query.order(dsl::license.desc()),
            },
            WorkField::CopyrightHolder => match order.direction {
                Direction::Asc => query.order(dsl::copyright_holder.asc()),
                Direction::Desc => query.order(dsl::copyright_holder.desc()),
            },
            WorkField::LandingPage => match order.direction {
                Direction::Asc => query.order(dsl::landing_page.asc()),
                Direction::Desc => query.order(dsl::landing_page.desc()),
            },
            WorkField::Lccn => match order.direction {
                Direction::Asc => query.order(dsl::lccn.asc()),
                Direction::Desc => query.order(dsl::lccn.desc()),
            },
            WorkField::Oclc => match order.direction {
                Direction::Asc => query.order(dsl::oclc.asc()),
                Direction::Desc => query.order(dsl::oclc.desc()),
            },
            WorkField::ShortAbstract => match order.direction {
                Direction::Asc => query.order(dsl::short_abstract.asc()),
                Direction::Desc => query.order(dsl::short_abstract.desc()),
            },
            WorkField::LongAbstract => match order.direction {
                Direction::Asc => query.order(dsl::long_abstract.asc()),
                Direction::Desc => query.order(dsl::long_abstract.desc()),
            },
            WorkField::GeneralNote => match order.direction {
                Direction::Asc => query.order(dsl::general_note.asc()),
                Direction::Desc => query.order(dsl::general_note.desc()),
            },
            WorkField::BibliographyNote => match order.direction {
                Direction::Asc => query.order(dsl::bibliography_note.asc()),
                Direction::Desc => query.order(dsl::bibliography_note.desc()),
            },
            WorkField::Toc => match order.direction {
                Direction::Asc => query.order(dsl::toc.asc()),
                Direction::Desc => query.order(dsl::toc.desc()),
            },
            WorkField::CoverUrl => match order.direction {
                Direction::Asc => query.order(dsl::cover_url.asc()),
                Direction::Desc => query.order(dsl::cover_url.desc()),
            },
            WorkField::CoverCaption => match order.direction {
                Direction::Asc => query.order(dsl::cover_caption.asc()),
                Direction::Desc => query.order(dsl::cover_caption.desc()),
            },
            WorkField::CreatedAt => match order.direction {
                Direction::Asc => query.order(dsl::created_at.asc()),
                Direction::Desc => query.order(dsl::created_at.desc()),
            },
            WorkField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(dsl::updated_at.asc()),
                Direction::Desc => query.order(dsl::updated_at.desc()),
            },
            WorkField::UpdatedAtWithRelations => match order.direction {
                Direction::Asc => query.order(dsl::updated_at_with_relations.asc()),
                Direction::Desc => query.order(dsl::updated_at_with_relations.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::imprint_id.eq(pid));
        }
        if !work_types.is_empty() {
            query = query.filter(dsl::work_type.eq_any(work_types));
        }
        if !work_statuses.is_empty() {
            query = query.filter(dsl::work_status.eq_any(work_statuses));
        }
        if let Some(updated) = updated_at_with_relations {
            match updated.expression {
                Expression::GreaterThan => {
                    query = query.filter(dsl::updated_at_with_relations.gt(updated.timestamp))
                }
                Expression::LessThan => {
                    query = query.filter(dsl::updated_at_with_relations.lt(updated.timestamp))
                }
            }
        }
        if let Some(filter) = filter {
            query = query.filter(
                dsl::full_title
                    .ilike(format!("%{filter}%"))
                    .or(dsl::doi.ilike(format!("%{filter}%")))
                    .or(dsl::reference.ilike(format!("%{filter}%")))
                    .or(dsl::short_abstract.ilike(format!("%{filter}%")))
                    .or(dsl::long_abstract.ilike(format!("%{filter}%")))
                    .or(dsl::landing_page.ilike(format!("%{filter}%"))),
            );
        }
        query
            .then_order_by(dsl::work_id)
            .limit(limit.into())
            .offset(offset.into())
            .load::<Work>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        work_types: Vec<Self::FilterParameter1>,
        work_statuses: Vec<Self::FilterParameter2>,
        updated_at_with_relations: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::work::dsl;
        let mut connection = db.get()?;
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if !work_types.is_empty() {
            query = query.filter(dsl::work_type.eq_any(work_types));
        }
        if !work_statuses.is_empty() {
            query = query.filter(dsl::work_status.eq_any(work_statuses));
        }
        if let Some(updated) = updated_at_with_relations {
            match updated.expression {
                Expression::GreaterThan => {
                    query = query.filter(dsl::updated_at_with_relations.gt(updated.timestamp))
                }
                Expression::LessThan => {
                    query = query.filter(dsl::updated_at_with_relations.lt(updated.timestamp))
                }
            }
        }
        if let Some(filter) = filter {
            query = query.filter(
                dsl::full_title
                    .ilike(format!("%{filter}%"))
                    .or(dsl::doi.ilike(format!("%{filter}%")))
                    .or(dsl::reference.ilike(format!("%{filter}%")))
                    .or(dsl::short_abstract.ilike(format!("%{filter}%")))
                    .or(dsl::long_abstract.ilike(format!("%{filter}%")))
                    .or(dsl::landing_page.ilike(format!("%{filter}%"))),
            );
        }

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        let imprint = crate::model::imprint::Imprint::from_id(db, &self.imprint_id)?;
        <crate::model::imprint::Imprint as Crud>::publisher_id(&imprint, db)
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
