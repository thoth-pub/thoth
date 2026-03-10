use super::{
    Imprint, ImprintField, ImprintHistory, ImprintOrderBy, NewImprint, NewImprintHistory,
    PatchImprint,
};
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{imprint, imprint_history};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;
    type OrderByEntity = ImprintOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.imprint_id
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
    ) -> ThothResult<Vec<Imprint>> {
        use crate::schema::imprint::dsl::*;
        let mut connection = db.get()?;
        let mut query = imprint.into_boxed();

        query = match order.field {
            ImprintField::ImprintId => apply_directional_order!(query, order.direction, order, imprint_id),
            ImprintField::ImprintName => apply_directional_order!(query, order.direction, order, imprint_name),
            ImprintField::ImprintUrl => apply_directional_order!(query, order.direction, order, imprint_url),
            ImprintField::CrossmarkDoi => apply_directional_order!(query, order.direction, order, crossmark_doi),
            ImprintField::CreatedAt => apply_directional_order!(query, order.direction, order, created_at),
            ImprintField::UpdatedAt => apply_directional_order!(query, order.direction, order, updated_at),
        };
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(publisher_id.eq(pid));
        }
        if let Some(filter) = filter {
            query = query.filter(
                imprint_name
                    .ilike(format!("%{filter}%"))
                    .or(imprint_url.ilike(format!("%{filter}%"))),
            );
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Imprint>(&mut connection)
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
        use crate::schema::imprint::dsl::*;
        let mut connection = db.get()?;
        let mut query = imprint.into_boxed();
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            query = query.filter(
                imprint_name
                    .ilike(format!("%{filter}%"))
                    .or(imprint_url.ilike(format!("%{filter}%"))),
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

    crud_methods!(imprint::table, imprint::dsl::imprint);
}

publisher_id_impls!(Imprint, NewImprint, PatchImprint, |s, _db| {
    Ok(s.publisher_id)
});

impl HistoryEntry for Imprint {
    type NewHistoryEntity = NewImprintHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            imprint_id: self.imprint_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewImprintHistory {
    type MainEntity = ImprintHistory;

    db_insert!(imprint_history::table);
}
