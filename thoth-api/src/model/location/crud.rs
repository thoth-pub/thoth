use super::{
    Location, LocationField, LocationHistory, LocationOrderBy, LocationPlatform, NewLocation,
    NewLocationHistory, PatchLocation,
};
use crate::db_insert;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{location, location_history};
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Location {
    type NewEntity = NewLocation;
    type PatchEntity = PatchLocation;
    type OrderByEntity = LocationOrderBy;
    type FilterParameter1 = LocationPlatform;
    type FilterParameter2 = ();
    type FilterParameter3 = ();

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
        location_platforms: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Location>> {
        use crate::schema::location::dsl::*;
        let mut connection = db.get()?;
        let mut query =
            location
                .inner_join(crate::schema::publication::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::imprint::table),
                ))
                .select(crate::schema::location::all_columns)
                .into_boxed();

        query = match order.field {
            LocationField::LocationId => match order.direction {
                Direction::Asc => query.order(location_id.asc()),
                Direction::Desc => query.order(location_id.desc()),
            },
            LocationField::PublicationId => match order.direction {
                Direction::Asc => query.order(publication_id.asc()),
                Direction::Desc => query.order(publication_id.desc()),
            },
            LocationField::LandingPage => match order.direction {
                Direction::Asc => query.order(landing_page.asc()),
                Direction::Desc => query.order(landing_page.desc()),
            },
            LocationField::FullTextUrl => match order.direction {
                Direction::Asc => query.order(full_text_url.asc()),
                Direction::Desc => query.order(full_text_url.desc()),
            },
            LocationField::LocationPlatform => match order.direction {
                Direction::Asc => query.order(location_platform.asc()),
                Direction::Desc => query.order(location_platform.desc()),
            },
            LocationField::Canonical => match order.direction {
                Direction::Asc => query.order(canonical.asc()),
                Direction::Desc => query.order(canonical.desc()),
            },
            LocationField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            LocationField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(publication_id.eq(pid));
        }
        if !location_platforms.is_empty() {
            query = query.filter(location_platform.eq_any(location_platforms));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Location>(&mut connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        location_platforms: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::location::dsl::*;
        let mut connection = db.get()?;
        let mut query = location.into_boxed();
        if !location_platforms.is_empty() {
            query = query.filter(location_platform.eq_any(location_platforms));
        }
        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        match query.count().get_result::<i64>(&mut connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::publication::Publication::from_id(db, &self.publication_id)?.publisher_id(db)
    }

    // `crud_methods!` cannot be used for update(), because we need to execute multiple statements
    // in the same transaction for changing a non-canonical location to canonical.
    // These functions recreate the `crud_methods!` logic.
    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        let mut connection = db.get()?;
        match location::table
            .find(entity_id)
            .get_result::<Self>(&mut connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn create(db: &crate::db::PgPool, data: &NewLocation) -> ThothResult<Self> {
        let mut connection = db.get()?;

        connection.transaction(|connection| {
            diesel::insert_into(location::table)
                .values(data)
                .get_result::<Self>(connection)
                .map_err(|e| e.into())
        })
    }

    fn update(
        &self,
        db: &crate::db::PgPool,
        data: &PatchLocation,
        account_id: &Uuid,
    ) -> ThothResult<Self> {
        let mut connection = db.get()?;
        let update_result: Result<Self, ThothError>;
        // if changes to a location don't change its canonical or non-canonical status, only update that location.
        if data.canonical == self.canonical {
            update_result = connection.transaction(|connection| {
                diesel::update(location::table.find(&self.location_id))
                    .set(data)
                    .get_result::<Self>(connection)
                    .map_err(ThothError::from)
            });
        // trying to change canonical location to non-canonical results in error.
        } else if self.canonical && (data.canonical != self.canonical) {
            return Err(ThothError::CanonicalLocationError);
        // if user changes a non-canonical location to canonical, perform two simultaneous updates:
        // change the former canonical location to non-canonical, the former non-canonical location to canonical
        } else {
            let canonical_location = self.get_canonical_location(db).map_err(ThothError::from)?;

            let old_canonical_location = PatchLocation {
                location_id: canonical_location.location_id,
                publication_id: canonical_location.publication_id,
                landing_page: canonical_location.landing_page.clone(),
                full_text_url: canonical_location.full_text_url.clone(),
                location_platform: canonical_location.location_platform.clone(),
                canonical: false,
            };
            update_result = connection.transaction(|connection| {
                // Update the currently canonical location to non-canonical
                diesel::update(location::table.find(&old_canonical_location.location_id.clone()))
                    .set(old_canonical_location)
                    .execute(connection)?;
                diesel::update(location::table.find(&self.location_id))
                    // Update the data from the currently non-canonical location to make it canonical,
                    // along with any other changes from PatchLocation
                    .set(data)
                    .get_result::<Self>(connection)
                    .map_err(ThothError::from)
            });
        }

        match update_result {
            Ok(l) => {
                let mut connection = db.get()?;
                match self.new_history_entry(account_id).insert(&mut connection) {
                    Ok(_) => Ok(l),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        let mut connection = db.get()?;
        connection.transaction(|connection| {
            match diesel::delete(location::table.find(self.location_id)).execute(connection) {
                Ok(_) => Ok(self),
                Err(e) => Err(ThothError::from(e)),
            }
        })
    }
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
        use crate::schema::location::dsl::*;
        use diesel::prelude::*;

        let mut connection = db.get()?;
        let canonical_count = location
            .filter(publication_id.eq(self.publication_id))
            .filter(canonical)
            .count()
            .get_result::<i64>(&mut connection)
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

    pub fn canonical_record_complete(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        location_canonical_record_complete(
            self.publication_id,
            &self.landing_page,
            &self.full_text_url,
            db,
        )
    }
}

impl Location {
    pub fn get_canonical_location(&self, db: &crate::db::PgPool) -> ThothResult<Location> {
        let mut connection = db.get()?;
        let canonical_location = crate::schema::location::table
            .filter(crate::schema::location::publication_id.eq(self.publication_id))
            .filter(crate::schema::location::canonical.eq(true))
            .first::<Location>(&mut connection)
            .expect("Error loading canonical location for publication");
        Ok(canonical_location)
    }
}

impl PatchLocation {
    pub fn canonical_record_complete(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        location_canonical_record_complete(
            self.publication_id,
            &self.landing_page,
            &self.full_text_url,
            db,
        )
    }
}

fn location_canonical_record_complete(
    publication_id: Uuid,
    landing_page: &Option<String>,
    full_text_url: &Option<String>,
    db: &crate::db::PgPool,
) -> ThothResult<()> {
    // If a canonical location has both the possible URLs, it is always complete.
    if landing_page.is_some() && full_text_url.is_some() {
        Ok(())
    } else {
        use crate::model::publication::PublicationType;
        use diesel::prelude::*;

        let mut connection = db.get()?;
        let publication_type = crate::schema::publication::table
            .select(crate::schema::publication::publication_type)
            .filter(crate::schema::publication::publication_id.eq(publication_id))
            .first::<PublicationType>(&mut connection)
            .expect("Error loading publication type for location");
        // If a canonical location's publication is of a digital type,
        // it must have both the possible URLs to count as complete.
        if publication_type != PublicationType::Hardback
            && publication_type != PublicationType::Paperback
        {
            Err(ThothError::LocationUrlError)
        } else {
            // For non-digital types, at least one URL must be present,
            // but exceptions to this will be caught at the database level.
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
