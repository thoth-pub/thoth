use super::{CurrencyCode, NewPrice, NewPriceHistory, PatchPrice, Price, PriceField, PriceHistory};
use crate::graphql::model::PriceOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{price, price_history};
use crate::{crud_methods, db_insert};
use diesel::dsl::any;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Price {
    type NewEntity = NewPrice;
    type PatchEntity = PatchPrice;
    type OrderByEntity = PriceOrderBy;
    type FilterParameter1 = CurrencyCode;
    type FilterParameter2 = ();

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
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Price>> {
        use crate::schema::price::dsl;
        let connection = db.get().unwrap();
        let mut query =
            dsl::price
                .inner_join(crate::schema::publication::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::imprint::table),
                ))
                .select((
                    dsl::price_id,
                    dsl::publication_id,
                    dsl::currency_code,
                    dsl::unit_price,
                    dsl::created_at,
                    dsl::updated_at,
                ))
                .into_boxed();

        match order.field {
            PriceField::PriceId => match order.direction {
                Direction::Asc => query = query.order(dsl::price_id.asc()),
                Direction::Desc => query = query.order(dsl::price_id.desc()),
            },
            PriceField::PublicationId => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_id.asc()),
                Direction::Desc => query = query.order(dsl::publication_id.desc()),
            },
            PriceField::CurrencyCode => match order.direction {
                Direction::Asc => query = query.order(dsl::currency_code.asc()),
                Direction::Desc => query = query.order(dsl::currency_code.desc()),
            },
            PriceField::UnitPrice => match order.direction {
                Direction::Asc => query = query.order(dsl::unit_price.asc()),
                Direction::Desc => query = query.order(dsl::unit_price.desc()),
            },
            PriceField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            PriceField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq(any(publishers)));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::publication_id.eq(pid));
        }
        if !currency_codes.is_empty() {
            query = query.filter(dsl::currency_code.eq(any(currency_codes)));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Price>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        currency_codes: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::price::dsl;
        let connection = db.get().unwrap();
        let mut query = dsl::price.into_boxed();
        if !currency_codes.is_empty() {
            query = query.filter(dsl::currency_code.eq(any(currency_codes)));
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
        crate::model::publication::Publication::from_id(db, &self.publication_id)?.publisher_id(db)
    }

    crud_methods!(price::table, price::dsl::price);
}

impl HistoryEntry for Price {
    type NewHistoryEntity = NewPriceHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            price_id: self.price_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewPriceHistory {
    type MainEntity = PriceHistory;

    db_insert!(price_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_pk() {
        let price: Price = Default::default();
        assert_eq!(price.pk(), price.price_id);
    }

    #[test]
    fn test_new_price_history_from_price() {
        let price: Price = Default::default();
        let account_id: Uuid = Default::default();
        let new_price_history = price.new_history_entry(&account_id);
        assert_eq!(new_price_history.price_id, price.price_id);
        assert_eq!(new_price_history.account_id, account_id);
        assert_eq!(
            new_price_history.data,
            serde_json::Value::String(serde_json::to_string(&price).unwrap())
        );
    }
}
