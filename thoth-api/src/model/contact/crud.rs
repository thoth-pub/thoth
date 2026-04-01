use super::{
    Contact, ContactField, ContactHistory, ContactOrderBy, ContactType, NewContact,
    NewContactHistory, PatchContact,
};
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
            ContactField::ContactId => {
                apply_directional_order!(query, order.direction, order, contact_id)
            }
            ContactField::PublisherId => {
                apply_directional_order!(query, order.direction, order, publisher_id)
            }
            ContactField::ContactType => {
                apply_directional_order!(query, order.direction, order, contact_type)
            }
            ContactField::Email => apply_directional_order!(query, order.direction, order, email),
            ContactField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            ContactField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
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
