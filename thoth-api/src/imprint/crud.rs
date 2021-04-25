use super::model::{Imprint, ImprintHistory, NewImprint, NewImprintHistory, PatchImprint};
use crate::graphql::utils::Direction;
use crate::imprint::model::{ImprintField, ImprintOrderBy};
pub use crate::model::Crud;
use crate::model::{DbInsert, HistoryEntry};
use crate::schema::{imprint, imprint_history};
use crate::{crud_methods, db_insert};

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;
    type OrderByEntity = ImprintOrderBy;
    type OptionalParameter = ();

    fn pk(&self) -> uuid::Uuid {
        self.imprint_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: String,
        order: Self::OrderByEntity,
        publishers: Vec<uuid::Uuid>,
        parent_id: Option<uuid::Uuid>,
        _filter_param: Option<Self::OptionalParameter>,
    ) -> crate::errors::ThothResult<Vec<Imprint>> {
        use crate::schema::imprint::dsl::*;
        use diesel::{
            BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
            RunQueryDsl,
        };
        let connection = db.get().unwrap();

        let mut query = imprint.into_boxed();

        match order.field {
            ImprintField::ImprintId => match order.direction {
                Direction::Asc => query = query.order(imprint_id.asc()),
                Direction::Desc => query = query.order(imprint_id.desc()),
            },
            ImprintField::ImprintName => match order.direction {
                Direction::Asc => query = query.order(imprint_name.asc()),
                Direction::Desc => query = query.order(imprint_name.desc()),
            },
            ImprintField::ImprintUrl => match order.direction {
                Direction::Asc => query = query.order(imprint_url.asc()),
                Direction::Desc => query = query.order(imprint_url.desc()),
            },
            ImprintField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            ImprintField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        if let Some(pid) = parent_id {
            query = query.filter(publisher_id.eq(pid));
        }
        // Ordering and construction of filters is important here: result needs to be
        // `WHERE (x = $1 [OR x = $2...]) AND (y ILIKE $3 [OR z ILIKE $3...])`.
        // Interchanging .filter, .or, and .or_filter would result in different bracketing.
        for pub_id in publishers {
            query = query.or_filter(publisher_id.eq(pub_id));
        }
        match query
            .filter(
                imprint_name
                    .ilike(format!("%{}%", filter))
                    .or(imprint_url.ilike(format!("%{}%", filter))),
            )
            .limit(limit.into())
            .offset(offset.into())
            .load::<Imprint>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(crate::errors::ThothError::from(e)),
        }
    }

    crud_methods!(imprint::table, imprint::dsl::imprint, ImprintHistory);
}

impl HistoryEntry for ImprintHistory {
    type MainEntity = Imprint;
    type NewEntity = NewImprintHistory;

    fn new(entity: &Self::MainEntity, account_id: &uuid::Uuid) -> Self::NewEntity {
        Self::NewEntity {
            imprint_id: entity.imprint_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&entity).unwrap()),
        }
    }
}

impl DbInsert for NewImprintHistory {
    type MainEntity = ImprintHistory;

    db_insert!(imprint_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Default for Imprint {
        fn default() -> Self {
            Imprint {
                imprint_id: Default::default(),
                publisher_id: Default::default(),
                imprint_name: Default::default(),
                imprint_url: Default::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }
    }

    #[test]
    fn test_imprint_pk() {
        let imprint: Imprint = Default::default();
        assert_eq!(imprint.pk(), imprint.imprint_id);
    }

    #[test]
    fn test_new_imprint_history_from_imprint() {
        let imprint: Imprint = Default::default();
        let account_id: uuid::Uuid = Default::default();
        let new_imprint_history = ImprintHistory::new(&imprint, &account_id);
        assert_eq!(new_imprint_history.imprint_id, imprint.imprint_id);
        assert_eq!(new_imprint_history.account_id, account_id);
        assert_eq!(
            new_imprint_history.data,
            serde_json::Value::String(serde_json::to_string(&imprint).unwrap())
        );
    }
}
