use super::{
    NewPublisher, NewPublisherHistory, PatchPublisher, Publisher, PublisherField, PublisherHistory,
    PublisherOrderBy,
};
use crate::db::PgPool;
use crate::model::publisher_distribution_platform::DistributionPlatform;
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
            PublisherField::PublisherId => {
                apply_directional_order!(query, order.direction, order, publisher_id)
            }
            PublisherField::PublisherName => {
                apply_directional_order!(query, order.direction, order, publisher_name)
            }
            PublisherField::PublisherShortname => {
                apply_directional_order!(query, order.direction, order, publisher_shortname)
            }
            PublisherField::PublisherUrl => {
                apply_directional_order!(query, order.direction, order, publisher_url)
            }
            PublisherField::ZitadelId => {
                apply_directional_order!(query, order.direction, order, zitadel_id)
            }
            PublisherField::AccessibilityStatement => {
                apply_directional_order!(query, order.direction, order, accessibility_statement)
            }
            PublisherField::AccessibilityReportUrl => {
                apply_directional_order!(query, order.direction, order, accessibility_report_url)
            }
            PublisherField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            PublisherField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
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
    /// Publishers with an **enabled** assignment for `platform`.
    ///
    /// One set-based join query. The composite assignment primary key
    /// `(publisher_id, platform)` plus the platform equality means a publisher
    /// can match at most once, so no duplicate row is possible. Ordering is the
    /// requested publisher order followed by a mandatory `publisher_id ASC`
    /// tie-breaker, which makes offset/limit pagination deterministic even when
    /// publisher names collide. When the primary sort field is already
    /// `publisher_id` the tie-breaker is redundant but harmless, and is kept so
    /// the deterministic ordering rule holds uniformly.
    pub fn all_by_distribution_platform(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        order: PublisherOrderBy,
        platform: DistributionPlatform,
    ) -> ThothResult<Vec<Publisher>> {
        use crate::schema::publisher::dsl::*;
        use crate::schema::publisher_distribution_platform::dsl as assignment;
        let mut connection = db.get()?;
        let mut query = publisher
            .inner_join(crate::schema::publisher_distribution_platform::table)
            .filter(assignment::platform.eq(platform))
            .filter(assignment::enabled.eq(true))
            .select(crate::schema::publisher::all_columns)
            .into_boxed();

        query = match order.field {
            PublisherField::PublisherId => {
                apply_directional_order!(query, order.direction, order, publisher_id, publisher_id)
            }
            PublisherField::PublisherName => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher_name,
                    publisher_id
                )
            }
            PublisherField::PublisherShortname => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    publisher_shortname,
                    publisher_id
                )
            }
            PublisherField::PublisherUrl => {
                apply_directional_order!(query, order.direction, order, publisher_url, publisher_id)
            }
            PublisherField::ZitadelId => {
                apply_directional_order!(query, order.direction, order, zitadel_id, publisher_id)
            }
            PublisherField::AccessibilityStatement => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    accessibility_statement,
                    publisher_id
                )
            }
            PublisherField::AccessibilityReportUrl => {
                apply_directional_order!(
                    query,
                    order.direction,
                    order,
                    accessibility_report_url,
                    publisher_id
                )
            }
            PublisherField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at, publisher_id)
            }
            PublisherField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at, publisher_id)
            }
        };

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publisher>(&mut connection)
            .map_err(Into::into)
    }

    /// The number of publishers with an **enabled** assignment for `platform`.
    ///
    /// This counts exactly the population returned by
    /// [`Self::all_by_distribution_platform`] before pagination.
    pub fn count_by_distribution_platform(
        db: &crate::db::PgPool,
        platform: DistributionPlatform,
    ) -> ThothResult<i32> {
        use crate::schema::publisher::dsl::*;
        use crate::schema::publisher_distribution_platform::dsl as assignment;
        let mut connection = db.get()?;

        // See the `Crud::count` note on the i64 -> i32 conversion.
        publisher
            .inner_join(crate::schema::publisher_distribution_platform::table)
            .filter(assignment::platform.eq(platform))
            .filter(assignment::enabled.eq(true))
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

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
