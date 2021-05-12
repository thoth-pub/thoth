use super::model::{
    NewPublisher, NewPublisherHistory, PatchPublisher, Publisher, PublisherField, PublisherHistory,
    PublisherOrderBy,
};
use crate::errors::{ThothError, ThothResult};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{publisher, publisher_history};
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};

impl Crud for Publisher {
    type NewEntity = NewPublisher;
    type PatchEntity = PatchPublisher;
    type OrderByEntity = PublisherOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> uuid::Uuid {
        self.publisher_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<uuid::Uuid>,
        _: Option<uuid::Uuid>,
        _: Option<uuid::Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Publisher>> {
        use crate::schema::publisher::dsl::*;
        let connection = db.get().unwrap();
        let mut query = publisher.into_boxed();

        match order.field {
            PublisherField::PublisherId => match order.direction {
                Direction::Asc => query = query.order(publisher_id.asc()),
                Direction::Desc => query = query.order(publisher_id.desc()),
            },
            PublisherField::PublisherName => match order.direction {
                Direction::Asc => query = query.order(publisher_name.asc()),
                Direction::Desc => query = query.order(publisher_name.desc()),
            },
            PublisherField::PublisherShortname => match order.direction {
                Direction::Asc => query = query.order(publisher_shortname.asc()),
                Direction::Desc => query = query.order(publisher_shortname.desc()),
            },
            PublisherField::PublisherUrl => match order.direction {
                Direction::Asc => query = query.order(publisher_url.asc()),
                Direction::Desc => query = query.order(publisher_url.desc()),
            },
            PublisherField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            PublisherField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        // This loop must appear before any other filter statements, as it takes advantage of
        // the behaviour of `or_filter` being equal to `filter` when no other filters are present yet.
        // Result needs to be `WHERE (x = $1 [OR x = $2...]) AND ([...])` - note bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        if let Some(filter) = filter {
            query = query.filter(
                publisher_name
                    .ilike(format!("%{}%", filter))
                    .or(publisher_shortname.ilike(format!("%{}%", filter))),
            );
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publisher>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<uuid::Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::publisher::dsl::*;
        let connection = db.get().unwrap();
        let mut query = publisher.into_boxed();
        // This loop must appear before any other filter statements, as it takes advantage of
        // the behaviour of `or_filter` being equal to `filter` when no other filters are present yet.
        // Result needs to be `WHERE (x = $1 [OR x = $2...]) AND ([...])` - note bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        if let Some(filter) = filter {
            query = query.filter(
                publisher_name
                    .ilike(format!("%{}%", filter))
                    .or(publisher_shortname.ilike(format!("%{}%", filter))),
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

    fn publisher_id(&self, _db: &crate::db::PgPool) -> uuid::Uuid {
        self.pk()
    }

    crud_methods!(publisher::table, publisher::dsl::publisher);
}

impl HistoryEntry for Publisher {
    type NewHistoryEntity = NewPublisherHistory;

    fn new_history_entry(&self, account_id: &uuid::Uuid) -> Self::NewHistoryEntity {
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
        let account_id: uuid::Uuid = Default::default();
        let new_publisher_history = publisher.new_history_entry(&account_id);
        assert_eq!(new_publisher_history.publisher_id, publisher.publisher_id);
        assert_eq!(new_publisher_history.account_id, account_id);
        assert_eq!(
            new_publisher_history.data,
            serde_json::Value::String(serde_json::to_string(&publisher).unwrap())
        );
    }
}
