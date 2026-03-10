use super::{CurrencyCode, NewPrice, NewPriceHistory, PatchPrice, Price, PriceField, PriceHistory};
use crate::graphql::types::inputs::PriceOrderBy;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{price, price_history};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Price {
    type NewEntity = NewPrice;
    type PatchEntity = PatchPrice;
    type OrderByEntity = PriceOrderBy;
    type FilterParameter1 = CurrencyCode;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.price_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        _: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        currency_codes: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Price>> {
        use crate::schema::price::dsl::*;
        let mut connection = db.get()?;
        let mut query =
            price
                .inner_join(crate::schema::publication::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::imprint::table),
                ))
                .select(crate::schema::price::all_columns)
                .into_boxed();

        query = match order.field {
            PriceField::PriceId => apply_directional_order!(query, order.direction, order, price_id),
            PriceField::PublicationId => apply_directional_order!(query, order.direction, order, publication_id),
            PriceField::CurrencyCode => apply_directional_order!(query, order.direction, order, currency_code),
            PriceField::UnitPrice => apply_directional_order!(query, order.direction, order, unit_price),
            PriceField::CreatedAt => apply_directional_order!(query, order.direction, order, created_at),
            PriceField::UpdatedAt => apply_directional_order!(query, order.direction, order, updated_at),
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(publication_id.eq(pid));
        }
        if !currency_codes.is_empty() {
            query = query.filter(currency_code.eq_any(currency_codes));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Price>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        currency_codes: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::price::dsl::*;
        let mut connection = db.get()?;
        let mut query = price.into_boxed();
        if !currency_codes.is_empty() {
            query = query.filter(currency_code.eq_any(currency_codes));
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

    crud_methods!(price::table, price::dsl::price);
}

publisher_id_impls!(Price, NewPrice, PatchPrice, |s, db| {
    crate::model::publication::Publication::from_id(db, &s.publication_id)?.publisher_id(db)
});

impl HistoryEntry for Price {
    type NewHistoryEntity = NewPriceHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            price_id: self.price_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewPriceHistory {
    type MainEntity = PriceHistory;

    db_insert!(price_history::table);
}
