use super::{
    Imprint, ImprintField, ImprintHistory, ImprintOrderBy, NewImprint, NewImprintHistory,
    PatchImprint,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{imprint, imprint_history};
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;
    type OrderByEntity = ImprintOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> Uuid {
        self.imprint_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Imprint>> {
        use crate::schema::imprint::dsl::*;
        let mut connection = db.get().unwrap();
        let mut query = imprint.into_boxed();

        query = match order.field {
            ImprintField::ImprintId => match order.direction {
                Direction::Asc => query.order(imprint_id.asc()),
                Direction::Desc => query.order(imprint_id.desc()),
            },
            ImprintField::ImprintName => match order.direction {
                Direction::Asc => query.order(imprint_name.asc()),
                Direction::Desc => query.order(imprint_name.desc()),
            },
            ImprintField::ImprintUrl => match order.direction {
                Direction::Asc => query.order(imprint_url.asc()),
                Direction::Desc => query.order(imprint_url.desc()),
            },
            ImprintField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            ImprintField::UpdatedAt => match order.direction {
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
        if let Some(filter) = filter {
            query = query.filter(
                imprint_name
                    .ilike(format!("%{}%", filter))
                    .or(imprint_url.ilike(format!("%{}%", filter))),
            );
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Imprint>(&mut connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::imprint::dsl::*;
        let mut connection = db.get().unwrap();
        let mut query = imprint.into_boxed();
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            query = query.filter(
                imprint_name
                    .ilike(format!("%{}%", filter))
                    .or(imprint_url.ilike(format!("%{}%", filter))),
            );
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

    fn publisher_id(&self, _db: &crate::db::PgPool) -> ThothResult<Uuid> {
        Ok(self.publisher_id)
    }

    crud_methods!(imprint::table, imprint::dsl::imprint);
}

impl HistoryEntry for Imprint {
    type NewHistoryEntity = NewImprintHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            imprint_id: self.imprint_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
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

    #[test]
    fn test_imprint_pk() {
        let imprint: Imprint = Default::default();
        assert_eq!(imprint.pk(), imprint.imprint_id);
    }

    #[test]
    fn test_new_imprint_history_from_imprint() {
        let imprint: Imprint = Default::default();
        let account_id: Uuid = Default::default();
        let new_imprint_history = imprint.new_history_entry(&account_id);
        assert_eq!(new_imprint_history.imprint_id, imprint.imprint_id);
        assert_eq!(new_imprint_history.account_id, account_id);
        assert_eq!(
            new_imprint_history.data,
            serde_json::Value::String(serde_json::to_string(&imprint).unwrap())
        );
    }
}
