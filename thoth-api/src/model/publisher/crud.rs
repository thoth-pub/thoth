use super::{
    NewPublisher, NewPublisherHistory, PatchPublisher, Publisher, PublisherField, PublisherHistory,
    PublisherOrderBy,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{publisher, publisher_history};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Publisher {
    type NewEntity = NewPublisher;
    type PatchEntity = PatchPublisher;
    type OrderByEntity = PublisherOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.publisher_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Publisher>> {
        use crate::schema::publisher::dsl::*;
        let mut connection = db.get()?;
        let mut query = publisher.into_boxed();

        query = match order.field {
            PublisherField::PublisherId => match order.direction {
                Direction::Asc => query.order(publisher_id.asc()),
                Direction::Desc => query.order(publisher_id.desc()),
            },
            PublisherField::PublisherName => match order.direction {
                Direction::Asc => query.order(publisher_name.asc()),
                Direction::Desc => query.order(publisher_name.desc()),
            },
            PublisherField::PublisherShortname => match order.direction {
                Direction::Asc => query.order(publisher_shortname.asc()),
                Direction::Desc => query.order(publisher_shortname.desc()),
            },
            PublisherField::PublisherUrl => match order.direction {
                Direction::Asc => query.order(publisher_url.asc()),
                Direction::Desc => query.order(publisher_url.desc()),
            },
            PublisherField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            PublisherField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            query = query.filter(
                publisher_name
                    .ilike(format!("%{filter}%"))
                    .or(publisher_shortname.ilike(format!("%{filter}%"))),
            );
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publisher>(&mut connection)
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
        use crate::schema::publisher::dsl::*;
        let mut connection = db.get()?;
        let mut query = publisher.into_boxed();
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            query = query.filter(
                publisher_name
                    .ilike(format!("%{filter}%"))
                    .or(publisher_shortname.ilike(format!("%{filter}%"))),
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

    fn publisher_id(&self, _db: &crate::db::PgPool) -> ThothResult<Uuid> {
        Ok(self.pk())
    }

    crud_methods!(publisher::table, publisher::dsl::publisher);
}

impl HistoryEntry for Publisher {
    type NewHistoryEntity = NewPublisherHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            publisher_id: self.publisher_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewPublisherHistory {
    type MainEntity = PublisherHistory;

    db_insert!(publisher_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publisher_pk() {
        let publisher: Publisher = Default::default();
        assert_eq!(publisher.pk(), publisher.publisher_id);
    }

    #[test]
    fn test_new_publisher_history_from_publisher() {
        let publisher: Publisher = Default::default();
        let account_id: Uuid = Default::default();
        let new_publisher_history = publisher.new_history_entry(&account_id);
        assert_eq!(new_publisher_history.publisher_id, publisher.publisher_id);
        assert_eq!(new_publisher_history.account_id, account_id);
        assert_eq!(
            new_publisher_history.data,
            serde_json::Value::String(serde_json::to_string(&publisher).unwrap())
        );
    }
}
