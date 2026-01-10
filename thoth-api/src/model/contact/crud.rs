use super::{
    Contact, ContactField, ContactHistory, ContactOrderBy, ContactType, NewContact,
    NewContactHistory, PatchContact,
};
use crate::graphql::inputs::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{contact, contact_history};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Contact {
    type NewEntity = NewContact;
    type PatchEntity = PatchContact;
    type OrderByEntity = ContactOrderBy;
    type FilterParameter1 = ContactType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.contact_id
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
        contact_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Contact>> {
        use crate::schema::contact::dsl::*;
        let mut connection = db.get()?;
        let mut query = contact.into_boxed();

        query = match order.field {
            ContactField::ContactId => match order.direction {
                Direction::Asc => query.order(contact_id.asc()),
                Direction::Desc => query.order(contact_id.desc()),
            },
            ContactField::PublisherId => match order.direction {
                Direction::Asc => query.order(publisher_id.asc()),
                Direction::Desc => query.order(publisher_id.desc()),
            },
            ContactField::ContactType => match order.direction {
                Direction::Asc => query.order(contact_type.asc()),
                Direction::Desc => query.order(contact_type.desc()),
            },
            ContactField::Email => match order.direction {
                Direction::Asc => query.order(email.asc()),
                Direction::Desc => query.order(email.desc()),
            },
            ContactField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            ContactField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if !contact_types.is_empty() {
            query = query.filter(contact_type.eq_any(contact_types));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(publisher_id.eq(pid));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contact>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        publishers: Vec<Uuid>,
        contact_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::contact::dsl::*;
        let mut connection = db.get()?;
        let mut query = contact.into_boxed();
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if !contact_types.is_empty() {
            query = query.filter(contact_type.eq_any(contact_types));
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

    crud_methods!(contact::table, contact::dsl::contact);
}

publisher_id_impls!(Contact, NewContact, PatchContact, |s, _db| {
    Ok(s.publisher_id)
});

impl HistoryEntry for Contact {
    type NewHistoryEntity = NewContactHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            contact_id: self.contact_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewContactHistory {
    type MainEntity = ContactHistory;

    db_insert!(contact_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_pk() {
        let contact: Contact = Default::default();
        assert_eq!(contact.pk(), contact.contact_id);
    }

    #[test]
    fn test_new_contact_history_from_contact() {
        let contact: Contact = Default::default();
        let user_id = "12345";
        let new_contact_history = contact.new_history_entry(user_id);
        assert_eq!(new_contact_history.contact_id, contact.contact_id);
        assert_eq!(new_contact_history.user_id, user_id);
        assert_eq!(
            new_contact_history.data,
            serde_json::Value::String(serde_json::to_string(&contact).unwrap())
        );
    }
}
