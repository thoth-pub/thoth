use super::{
    NewWebhook, NewWebhookHistory, PatchWebhook, Webhook, WebhookField, WebhookHistory,
    WebhookOrderBy,
};
use crate::event::model::EventType;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{webhook, webhook_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Webhook {
    type NewEntity = NewWebhook;
    type PatchEntity = PatchWebhook;
    type OrderByEntity = WebhookOrderBy;
    type FilterParameter1 = EventType;
    type FilterParameter2 = ();
    type FilterParameter3 = bool;

    fn pk(&self) -> Uuid {
        self.webhook_id
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
        event_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        published: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Webhook>> {
        use crate::schema::webhook::dsl::*;
        let mut connection = db.get()?;
        let mut query = webhook.into_boxed();

        query = match order.field {
            WebhookField::WebhookId => match order.direction {
                Direction::Asc => query.order(webhook_id.asc()),
                Direction::Desc => query.order(webhook_id.desc()),
            },
            WebhookField::PublisherId => match order.direction {
                Direction::Asc => query.order(publisher_id.asc()),
                Direction::Desc => query.order(publisher_id.desc()),
            },
            WebhookField::Endpoint => match order.direction {
                Direction::Asc => query.order(endpoint.asc()),
                Direction::Desc => query.order(endpoint.desc()),
            },
            WebhookField::Token => match order.direction {
                Direction::Asc => query.order(token.asc()),
                Direction::Desc => query.order(token.desc()),
            },
            WebhookField::IsPublished => match order.direction {
                Direction::Asc => query.order(is_published.asc()),
                Direction::Desc => query.order(is_published.desc()),
            },
            WebhookField::EventType => match order.direction {
                Direction::Asc => query.order(event_type.asc()),
                Direction::Desc => query.order(event_type.desc()),
            },
            WebhookField::Platform => match order.direction {
                Direction::Asc => query.order(platform.asc()),
                Direction::Desc => query.order(platform.desc()),
            },
            WebhookField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            WebhookField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(publisher_id.eq(pid));
        }
        if !event_types.is_empty() {
            query = query.filter(event_type.eq_any(event_types));
        }
        if let Some(publ) = published {
            query = query.filter(is_published.eq(publ));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Webhook>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        publishers: Vec<Uuid>,
        event_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        published: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::webhook::dsl::*;
        let mut connection = db.get()?;
        let mut query = webhook.into_boxed();
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if !event_types.is_empty() {
            query = query.filter(event_type.eq_any(event_types));
        }
        if let Some(publ) = published {
            query = query.filter(is_published.eq(publ));
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
        Ok(self.publisher_id)
    }

    crud_methods!(webhook::table, webhook::dsl::webhook);
}

impl HistoryEntry for Webhook {
    type NewHistoryEntity = NewWebhookHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            webhook_id: self.webhook_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewWebhookHistory {
    type MainEntity = WebhookHistory;

    db_insert!(webhook_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_pk() {
        let webhook: Webhook = Default::default();
        assert_eq!(webhook.pk(), webhook.webhook_id);
    }

    #[test]
    fn test_new_webhook_history_from_webhook() {
        let webhook: Webhook = Default::default();
        let account_id: Uuid = Default::default();
        let new_webhook_history = webhook.new_history_entry(&account_id);
        assert_eq!(new_webhook_history.webhook_id, webhook.webhook_id);
        assert_eq!(new_webhook_history.account_id, account_id);
        assert_eq!(
            new_webhook_history.data,
            serde_json::Value::String(serde_json::to_string(&webhook).unwrap())
        );
    }
}
