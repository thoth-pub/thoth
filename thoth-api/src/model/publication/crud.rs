use super::{
    NewPublication, NewPublicationHistory, PatchPublication, Publication, PublicationField,
    PublicationHistory, PublicationOrderBy, PublicationType,
};
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{publication, publication_history};
use diesel::{
    dsl::sql, sql_types::Text, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Publication {
    type NewEntity = NewPublication;
    type PatchEntity = PatchPublication;
    type OrderByEntity = PublicationOrderBy;
    type FilterParameter1 = PublicationType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.publication_id
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
        publication_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Publication>> {
        use crate::schema::publication::dsl::*;
        let mut connection = db.get()?;
        let mut query = publication
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::publication::all_columns)
            .into_boxed();

        query = match order.field {
            PublicationField::PublicationId => {
                apply_directional_order!(query, order.direction, order, publication_id)
            }
            PublicationField::PublicationType => {
                apply_directional_order!(query, order.direction, order, publication_type)
            }
            PublicationField::WorkId => {
                apply_directional_order!(query, order.direction, order, work_id)
            }
            PublicationField::Isbn => apply_directional_order!(query, order.direction, order, isbn),
            PublicationField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            PublicationField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
            PublicationField::WidthMm => {
                apply_directional_order!(query, order.direction, order, width_mm)
            }
            PublicationField::WidthIn => {
                apply_directional_order!(query, order.direction, order, width_in)
            }
            PublicationField::HeightMm => {
                apply_directional_order!(query, order.direction, order, height_mm)
            }
            PublicationField::HeightIn => {
                apply_directional_order!(query, order.direction, order, height_in)
            }
            PublicationField::DepthMm => {
                apply_directional_order!(query, order.direction, order, depth_mm)
            }
            PublicationField::DepthIn => {
                apply_directional_order!(query, order.direction, order, depth_in)
            }
            PublicationField::WeightG => {
                apply_directional_order!(query, order.direction, order, weight_g)
            }
            PublicationField::WeightOz => {
                apply_directional_order!(query, order.direction, order, weight_oz)
            }
            PublicationField::AccessibilityStandard => {
                apply_directional_order!(query, order.direction, order, accessibility_standard)
            }
            PublicationField::AccessibilityAdditionalStandard => apply_directional_order!(
                query,
                order.direction,
                order,
                accessibility_additional_standard
            ),
            PublicationField::AccessibilityException => {
                apply_directional_order!(query, order.direction, order, accessibility_exception)
            }
            PublicationField::AccessibilityReportUrl => {
                apply_directional_order!(query, order.direction, order, accessibility_report_url)
            }
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if !publication_types.is_empty() {
            query = query.filter(publication_type.eq_any(publication_types));
        }
        if let Some(filter) = filter {
            // ISBN field is nullable, so searching with an empty filter could fail
            if !filter.is_empty() {
                // Ignore ISBN hyphenation when searching
                query = query.filter(
                    sql::<Text>("replace(isbn, '-', '')")
                        .ilike(format!("%{}%", filter.replace("-", ""))),
                );
            }
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publication>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        publication_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::publication::dsl::*;
        let mut connection = db.get()?;
        let mut query = publication
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if !publication_types.is_empty() {
            query = query.filter(publication_type.eq_any(publication_types));
        }
        if let Some(filter) = filter {
            // ISBN field is nullable, so searching with an empty filter could fail
            if !filter.is_empty() {
                // Ignore ISBN hyphenation when searching
                query = query.filter(
                    sql::<Text>("replace(isbn, '-', '')")
                        .ilike(format!("%{}%", filter.replace("-", ""))),
                );
            }
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

    crud_methods!(publication::table, publication::dsl::publication);
}

publisher_id_impls!(Publication, NewPublication, PatchPublication, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for Publication {
    type NewHistoryEntity = NewPublicationHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            publication_id: self.publication_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewPublicationHistory {
    type MainEntity = PublicationHistory;

    db_insert!(publication_history::table);
}
