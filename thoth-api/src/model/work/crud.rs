use super::{
    NewWork, NewWorkHistory, PatchWork, Work, WorkField, WorkHistory, WorkOrderBy, WorkStatus,
    WorkType,
};
use crate::graphql::types::inputs::Expression;
use crate::graphql::types::inputs::TimeExpression;
use crate::model::work_relation::{RelationType, WorkRelation, WorkRelationOrderBy};
use crate::model::{Crud, DbInsert, Doi, HistoryEntry, PublisherId};
use crate::schema::{work, work_abstract, work_history, work_title};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, PgTextExpressionMethods,
    QueryDsl, RunQueryDsl,
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
        use diesel::{
            dsl::sql,
            sql_types::{Nullable, Text},
        };

        let mut connection = db.get()?;
        // Allow case-insensitive searching (DOIs in database may have mixed casing)
        let mut query = dsl::work
            .filter(sql::<Nullable<Text>>("lower(doi)").eq(doi.to_lowercase_string()))
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
        use crate::schema::{
            additional_resource, award, book_review, endorsement, publication, work,
            work_featured_video,
        };
        let mut connection = db.get()?;

        let isbn_count = publication::table
            .filter(publication::work_id.eq(self.work_id))
            .filter(publication::isbn.is_not_null())
            .count()
            .get_result::<i64>(&mut connection)
            .expect("Error loading publication ISBNs for work")
            .to_string()
            .parse::<i32>()
            .unwrap();

        if isbn_count > 0 {
            Err(ThothError::ChapterIsbnError)
        } else {
            let additional_resource_count = additional_resource::table
                .filter(additional_resource::work_id.eq(self.work_id))
                .count()
                .get_result::<i64>(&mut connection)
                .expect("Error loading additional resources for work");
            let award_count = award::table
                .filter(award::work_id.eq(self.work_id))
                .count()
                .get_result::<i64>(&mut connection)
                .expect("Error loading awards for work");
            let endorsement_count = endorsement::table
                .filter(endorsement::work_id.eq(self.work_id))
                .count()
                .get_result::<i64>(&mut connection)
                .expect("Error loading endorsements for work");
            let review_count = book_review::table
                .filter(book_review::work_id.eq(self.work_id))
                .count()
                .get_result::<i64>(&mut connection)
                .expect("Error loading reviews for work");
            let featured_video_count = work_featured_video::table
                .filter(work_featured_video::work_id.eq(self.work_id))
                .count()
                .get_result::<i64>(&mut connection)
                .expect("Error loading featured videos for work");
            let resources_description_count = work::table
                .filter(work::work_id.eq(self.work_id))
                .filter(work::resources_description.is_not_null())
                .count()
                .get_result::<i64>(&mut connection)
                .expect("Error loading resources description for work");

            if additional_resource_count > 0
                || award_count > 0
                || endorsement_count > 0
                || review_count > 0
                || featured_video_count > 0
                || resources_description_count > 0
            {
                Err(ThothError::ChapterBookMetadataError)
            } else {
                Ok(())
            }
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
    type FilterParameter4 = TimeExpression;

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
        publication_date: Option<Self::FilterParameter3>,
        updated_at_with_relations: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Work>> {
        use crate::schema::work::dsl;

        let mut connection = db.get()?;
        let mut query = dsl::work
            .inner_join(crate::schema::imprint::table)
            .select(crate::schema::work::all_columns)
            .into_boxed();

        query = match order.field {
            WorkField::WorkId => {
                apply_directional_order!(query, order.direction, order_by, dsl::work_id)
            }
            WorkField::WorkType => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::work_type,
                dsl::work_id
            ),
            WorkField::WorkStatus => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::work_status,
                dsl::work_id
            ),
            WorkField::FullTitle => {
                let canonical_full_title = work_title::table
                    .select(work_title::full_title.nullable())
                    .filter(work_title::work_id.eq(dsl::work_id))
                    .filter(work_title::canonical.eq(true))
                    .order(work_title::title_id.asc())
                    .limit(1)
                    .single_value();
                apply_directional_order!(
                    query,
                    order.direction,
                    order_by,
                    canonical_full_title,
                    dsl::work_id
                )
            }
            WorkField::Title => {
                let canonical_title = work_title::table
                    .select(work_title::title.nullable())
                    .filter(work_title::work_id.eq(dsl::work_id))
                    .filter(work_title::canonical.eq(true))
                    .order(work_title::title_id.asc())
                    .limit(1)
                    .single_value();
                apply_directional_order!(
                    query,
                    order.direction,
                    order_by,
                    canonical_title,
                    dsl::work_id
                )
            }
            WorkField::Subtitle => {
                let canonical_subtitle = work_title::table
                    .select(work_title::subtitle)
                    .filter(work_title::work_id.eq(dsl::work_id))
                    .filter(work_title::canonical.eq(true))
                    .order(work_title::title_id.asc())
                    .limit(1)
                    .single_value();
                apply_directional_order!(
                    query,
                    order.direction,
                    order_by,
                    canonical_subtitle,
                    dsl::work_id
                )
            }
            WorkField::Reference => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::reference,
                dsl::work_id
            ),
            WorkField::Edition => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::edition,
                dsl::work_id
            ),
            WorkField::Doi => {
                apply_directional_order!(query, order.direction, order_by, dsl::doi, dsl::work_id)
            }
            WorkField::PublicationDate => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::publication_date,
                dsl::work_id
            ),
            WorkField::WithdrawnDate => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::withdrawn_date,
                dsl::work_id
            ),
            WorkField::Place => {
                apply_directional_order!(query, order.direction, order_by, dsl::place, dsl::work_id)
            }
            WorkField::PageCount => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::page_count,
                dsl::work_id
            ),
            WorkField::PageBreakdown => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::page_breakdown,
                dsl::work_id
            ),
            WorkField::FirstPage => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::first_page,
                dsl::work_id
            ),
            WorkField::LastPage => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::last_page,
                dsl::work_id
            ),
            WorkField::PageInterval => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::page_interval,
                dsl::work_id
            ),
            WorkField::ImageCount => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::image_count,
                dsl::work_id
            ),
            WorkField::TableCount => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::table_count,
                dsl::work_id
            ),
            WorkField::AudioCount => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::audio_count,
                dsl::work_id
            ),
            WorkField::VideoCount => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::video_count,
                dsl::work_id
            ),
            WorkField::License => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::license,
                dsl::work_id
            ),
            WorkField::CopyrightHolder => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::copyright_holder,
                dsl::work_id
            ),
            WorkField::LandingPage => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::landing_page,
                dsl::work_id
            ),
            WorkField::Lccn => {
                apply_directional_order!(query, order.direction, order_by, dsl::lccn, dsl::work_id)
            }
            WorkField::Oclc => {
                apply_directional_order!(query, order.direction, order_by, dsl::oclc, dsl::work_id)
            }
            WorkField::ShortAbstract | WorkField::LongAbstract => {
                let canonical_abstract = work_abstract::table
                    .select(work_abstract::content.nullable())
                    .filter(work_abstract::work_id.eq(dsl::work_id))
                    .filter(work_abstract::canonical.eq(true))
                    .order(work_abstract::abstract_id.asc())
                    .limit(1)
                    .single_value();
                apply_directional_order!(
                    query,
                    order.direction,
                    order_by,
                    canonical_abstract,
                    dsl::work_id
                )
            }
            WorkField::GeneralNote => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::general_note,
                dsl::work_id
            ),
            WorkField::BibliographyNote => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::bibliography_note,
                dsl::work_id
            ),
            WorkField::Toc => {
                apply_directional_order!(query, order.direction, order_by, dsl::toc, dsl::work_id)
            }
            WorkField::ResourcesDescription => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::resources_description,
                dsl::work_id
            ),
            WorkField::CoverUrl => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::cover_url,
                dsl::work_id
            ),
            WorkField::CoverCaption => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::cover_caption,
                dsl::work_id
            ),
            WorkField::CreatedAt => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::created_at,
                dsl::work_id
            ),
            WorkField::UpdatedAt => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::updated_at,
                dsl::work_id
            ),
            WorkField::UpdatedAtWithRelations => apply_directional_order!(
                query,
                order.direction,
                order_by,
                dsl::updated_at_with_relations,
                dsl::work_id
            ),
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

        apply_time_filter!(
            query,
            dsl::publication_date,
            publication_date,
            |ts: crate::model::Timestamp| ts.0.date_naive()
        );
        apply_time_filter!(
            query,
            dsl::updated_at_with_relations,
            updated_at_with_relations,
            |ts: crate::model::Timestamp| ts.0
        );

        if let Some(filter) = filter {
            let title_work_ids = work_title::table
                .filter(work_title::full_title.ilike(format!("%{filter}%")))
                .select(work_title::work_id)
                .load::<Uuid>(&mut connection)?;

            let abstract_work_ids = work_abstract::table
                .filter(work_abstract::content.ilike(format!("%{filter}%")))
                .select(work_abstract::work_id)
                .load::<Uuid>(&mut connection)?;

            query = query.filter(
                dsl::doi
                    .ilike(format!("%{filter}%"))
                    .or(dsl::doi.ilike(format!("%{filter}%")))
                    .or(dsl::reference.ilike(format!("%{filter}%")))
                    .or(dsl::landing_page.ilike(format!("%{filter}%")))
                    .or(dsl::resources_description.ilike(format!("%{filter}%")))
                    .or(dsl::work_id
                        .eq_any(title_work_ids)
                        .or(dsl::work_id.eq_any(abstract_work_ids))),
            );
        }
        query
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
        publication_date: Option<Self::FilterParameter3>,
        updated_at_with_relations: Option<Self::FilterParameter4>,
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

        apply_time_filter!(
            query,
            dsl::publication_date,
            publication_date,
            |ts: crate::model::Timestamp| ts.0.date_naive()
        );
        apply_time_filter!(
            query,
            dsl::updated_at_with_relations,
            updated_at_with_relations,
            |ts: crate::model::Timestamp| ts.0
        );

        if let Some(filter) = filter {
            let title_work_ids = work_title::table
                .filter(work_title::full_title.ilike(format!("%{filter}%")))
                .select(work_title::work_id)
                .load::<Uuid>(&mut connection)?;

            let abstract_work_ids = work_abstract::table
                .filter(work_abstract::content.ilike(format!("%{filter}%")))
                .select(work_abstract::work_id)
                .load::<Uuid>(&mut connection)?;

            query = query.filter(
                dsl::doi
                    .ilike(format!("%{filter}%"))
                    .or(dsl::reference.ilike(format!("%{filter}%")))
                    .or(dsl::landing_page.ilike(format!("%{filter}%")))
                    .or(dsl::resources_description.ilike(format!("%{filter}%")))
                    .or(dsl::work_id.eq_any(title_work_ids))
                    .or(dsl::work_id.eq_any(abstract_work_ids)),
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

    crud_methods!(work::table, work::dsl::work);
}

publisher_id_impls!(Work, NewWork, PatchWork, |s, db| {
    let imprint = crate::model::imprint::Imprint::from_id(db, &s.imprint_id)?;
    <crate::model::imprint::Imprint as PublisherId>::publisher_id(&imprint, db)
});

impl HistoryEntry for Work {
    type NewHistoryEntity = NewWorkHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            work_id: self.work_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewWorkHistory {
    type MainEntity = WorkHistory;

    db_insert!(work_history::table);
}
