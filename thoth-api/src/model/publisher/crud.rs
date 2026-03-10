use super::{
    NewPublisher, NewPublisherHistory, PatchPublisher, Publisher, PublisherField, PublisherHistory,
    PublisherOrderBy,
};
use crate::db::PgPool;
use crate::model::{Crud, DbInsert, HistoryEntry, PublisherId};
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
            PublisherField::PublisherId => apply_directional_order!(query, order.direction, order, publisher_id),
            PublisherField::PublisherName => apply_directional_order!(query, order.direction, order, publisher_name),
            PublisherField::PublisherShortname => apply_directional_order!(query, order.direction, order, publisher_shortname),
            PublisherField::PublisherUrl => apply_directional_order!(query, order.direction, order, publisher_url),
            PublisherField::ZitadelId => apply_directional_order!(query, order.direction, order, zitadel_id),
            PublisherField::AccessibilityStatement => apply_directional_order!(query, order.direction, order, accessibility_statement),
            PublisherField::AccessibilityReportUrl => apply_directional_order!(query, order.direction, order, accessibility_report_url),
            PublisherField::CreatedAt => apply_directional_order!(query, order.direction, order, created_at),
            PublisherField::UpdatedAt => apply_directional_order!(query, order.direction, order, updated_at),
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

    crud_methods!(publisher::table, publisher::dsl::publisher);
}

impl Publisher {
    pub fn by_zitadel_ids(
        db: &crate::db::PgPool,
        org_ids: Vec<String>,
    ) -> ThothResult<Vec<Publisher>> {
        use crate::schema::publisher::dsl::*;

        if org_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut connection = db.get()?;
        let org_ids: Vec<Option<String>> = org_ids.into_iter().map(Some).collect();

        publisher
            .filter(zitadel_id.eq_any(org_ids))
            .load::<Publisher>(&mut connection)
            .map_err(Into::into)
    }
}

impl PublisherId for Publisher {
    fn publisher_id(&self, _db: &PgPool) -> ThothResult<Uuid> {
        Ok(self.publisher_id)
    }
}

impl PublisherId for PatchPublisher {
    fn publisher_id(&self, _db: &PgPool) -> ThothResult<Uuid> {
        Ok(self.publisher_id)
    }
}

impl HistoryEntry for Publisher {
    type NewHistoryEntity = NewPublisherHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            publisher_id: self.publisher_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewPublisherHistory {
    type MainEntity = PublisherHistory;

    db_insert!(publisher_history::table);
}
