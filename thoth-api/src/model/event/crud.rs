use super::{
    Event, EventType, NewEvent,
};
use crate::model::DbInsert;
use crate::schema::event;
use crate::db_insert;
use thoth_errors::{ThothError, ThothResult};

impl Event {
    fn pk(&self) -> Uuid {
        self.event_id
    }

    fn create(db: &crate::db::PgPool, data: &NewEvent) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::insert_into(event::table)
            .values(data)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn all(db: &crate::db::PgPool) -> ThothResult<Vec<Self>> {
        let mut connection = db.get()?;
        dsl::event
            .select(crate::schema::event::all_columns)
            .order(dsl::event_timestamp.desc())
            .then_order_by(dsl::event_id)
            .load::<Self>(&mut connection)
            .map_err(Into::into)
    }
}

impl DbInsert for NewEvent {
    type MainEntity = Event;

    db_insert!(event::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_pk() {
        let event: Event = Default::default();
        assert_eq!(event.pk(), event.event_id);
    }
}