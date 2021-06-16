use super::model::{
    Funder, FunderField, FunderHistory, FunderOrderBy, NewFunder, NewFunderHistory, PatchFunder,
};
use crate::errors::{ThothError, ThothResult};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{funder, funder_history};
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use uuid::Uuid;

impl Crud for Funder {
    type NewEntity = NewFunder;
    type PatchEntity = PatchFunder;
    type OrderByEntity = FunderOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> Uuid {
        self.funder_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        _: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Funder>> {
        use crate::schema::funder::dsl::*;
        let connection = db.get().unwrap();
        let mut query = funder.into_boxed();

        match order.field {
            FunderField::FunderId => match order.direction {
                Direction::Asc => query = query.order(funder_id.asc()),
                Direction::Desc => query = query.order(funder_id.desc()),
            },
            FunderField::FunderName => match order.direction {
                Direction::Asc => query = query.order(funder_name.asc()),
                Direction::Desc => query = query.order(funder_name.desc()),
            },
            FunderField::FunderDoi => match order.direction {
                Direction::Asc => query = query.order(funder_doi.asc()),
                Direction::Desc => query = query.order(funder_doi.desc()),
            },
            FunderField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            FunderField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        if let Some(filter) = filter {
            query = query.filter(
                funder_name
                    .ilike(format!("%{}%", filter))
                    .or(funder_doi.ilike(format!("%{}%", filter))),
            );
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Funder>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::funder::dsl::*;
        let connection = db.get().unwrap();
        let mut query = funder.into_boxed();
        if let Some(filter) = filter {
            query = query.filter(
                funder_name
                    .ilike(format!("%{}%", filter))
                    .or(funder_doi.ilike(format!("%{}%", filter))),
            );
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

    fn publisher_id(&self, _db: &crate::db::PgPool) -> ThothResult<Uuid> {
        Err(ThothError::InternalError(
            "Method publisher_id() is not supported for Funder objects".to_string(),
        ))
    }

    crud_methods!(funder::table, funder::dsl::funder);
}

impl HistoryEntry for Funder {
    type NewHistoryEntity = NewFunderHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            funder_id: self.funder_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewFunderHistory {
    type MainEntity = FunderHistory;

    db_insert!(funder_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_funder_pk() {
        let funder: Funder = Default::default();
        assert_eq!(funder.pk(), funder.funder_id);
    }

    #[test]
    fn test_new_funder_history_from_funder() {
        let funder: Funder = Default::default();
        let account_id: Uuid = Default::default();
        let new_funder_history = funder.new_history_entry(&account_id);
        assert_eq!(new_funder_history.funder_id, funder.funder_id);
        assert_eq!(new_funder_history.account_id, account_id);
        assert_eq!(
            new_funder_history.data,
            serde_json::Value::String(serde_json::to_string(&funder).unwrap())
        );
    }
}
