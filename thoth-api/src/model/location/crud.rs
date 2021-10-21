use super::{
    Location, LocationField, LocationHistory, LocationPlatform, NewLocation, NewLocationHistory,
    PatchLocation,
};
use crate::graphql::model::LocationOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{location, location_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Location {
    type NewEntity = NewLocation;
    type PatchEntity = PatchLocation;
    type OrderByEntity = LocationOrderBy;
    type FilterParameter1 = LocationPlatform;
    type FilterParameter2 = ();

    fn pk(&self) -> Uuid {
        self.location_id
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
        location_platform: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Location>> {
        use crate::schema::location::dsl;
        let connection = db.get().unwrap();
        let mut query =
            dsl::location
                .inner_join(crate::schema::publication::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::imprint::table),
                ))
                .select((
                    dsl::location_id,
                    dsl::publication_id,
                    dsl::landing_page,
                    dsl::full_text_url,
                    dsl::location_platform,
                    dsl::canonical,
                    dsl::created_at,
                    dsl::updated_at,
                ))
                .into_boxed();

        match order.field {
            LocationField::LocationId => match order.direction {
                Direction::Asc => query = query.order(dsl::location_id.asc()),
                Direction::Desc => query = query.order(dsl::location_id.desc()),
            },
            LocationField::PublicationId => match order.direction {
                Direction::Asc => query = query.order(dsl::publication_id.asc()),
                Direction::Desc => query = query.order(dsl::publication_id.desc()),
            },
            LocationField::LandingPage => match order.direction {
                Direction::Asc => query = query.order(dsl::landing_page.asc()),
                Direction::Desc => query = query.order(dsl::landing_page.desc()),
            },
            LocationField::FullTextUrl => match order.direction {
                Direction::Asc => query = query.order(dsl::full_text_url.asc()),
                Direction::Desc => query = query.order(dsl::full_text_url.desc()),
            },
            LocationField::LocationPlatform => match order.direction {
                Direction::Asc => query = query.order(dsl::location_platform.asc()),
                Direction::Desc => query = query.order(dsl::location_platform.desc()),
            },
            LocationField::Canonical => match order.direction {
                Direction::Asc => query = query.order(dsl::canonical.asc()),
                Direction::Desc => query = query.order(dsl::canonical.desc()),
            },
            LocationField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::created_at.asc()),
                Direction::Desc => query = query.order(dsl::created_at.desc()),
            },
            LocationField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(dsl::updated_at.asc()),
                Direction::Desc => query = query.order(dsl::updated_at.desc()),
            },
        }
        // This loop must appear before any other filter statements, as it takes advantage of
        // the behaviour of `or_filter` being equal to `filter` when no other filters are present yet.
        // Result needs to be `WHERE (x = $1 [OR x = $2...]) AND ([...])` - note bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::publication_id.eq(pid));
        }
        if let Some(loc_platform) = location_platform {
            query = query.filter(dsl::location_platform.eq(loc_platform));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Location>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        location_platform: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::location::dsl;
        let connection = db.get().unwrap();
        let mut query = dsl::location.into_boxed();
        if let Some(loc_platform) = location_platform {
            query = query.filter(dsl::location_platform.eq(loc_platform));
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

    crud_methods!(location::table, location::dsl::location);
}

impl HistoryEntry for Location {
    type NewHistoryEntity = NewLocationHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            location_id: self.location_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewLocationHistory {
    type MainEntity = LocationHistory;

    db_insert!(location_history::table);
}

impl NewLocation {
    pub fn can_be_non_canonical(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        use crate::schema::location::dsl;
        use diesel::prelude::*;

        let connection = db.get().unwrap();
        let canonical_count = dsl::location
            .filter(dsl::publication_id.eq(self.publication_id))
            .filter(dsl::canonical)
            .count()
            .get_result::<i64>(&connection)
            .expect("Error loading locations for publication")
            .to_string()
            .parse::<i32>()
            .unwrap();
        // A location can only be non-canonical if another location
        // marked as canonical exists for the same publication.
        if canonical_count == 0 {
            Err(ThothError::CanonicalLocationError)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_pk() {
        let location: Location = Default::default();
        assert_eq!(location.pk(), location.location_id);
    }

    #[test]
    fn test_new_location_history_from_location() {
        let location: Location = Default::default();
        let account_id: Uuid = Default::default();
        let new_location_history = location.new_history_entry(&account_id);
        assert_eq!(new_location_history.location_id, location.location_id);
        assert_eq!(new_location_history.account_id, account_id);
        assert_eq!(
            new_location_history.data,
            serde_json::Value::String(serde_json::to_string(&location).unwrap())
        );
    }
}
