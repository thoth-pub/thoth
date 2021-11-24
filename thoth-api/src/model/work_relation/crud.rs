use super::{
    NewWorkRelation, NewWorkRelationHistory, PatchWorkRelation, WorkRelation, WorkRelationField,
    WorkRelationHistory, WorkRelationOrderBy,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{work_relation, work_relation_history};
use crate::{crud_methods, db_insert};
use diesel::dsl::any;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for WorkRelation {
    type NewEntity = NewWorkRelation;
    type PatchEntity = PatchWorkRelation;
    type OrderByEntity = WorkRelationOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> Uuid {
        self.work_relation_id
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
        _: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<WorkRelation>> {
        use crate::schema::work_relation::dsl::*;
        let connection = db.get().unwrap();
        let mut query = work_relation
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                work_relation_id,
                relator_work_id,
                related_work_id,
                relation_type,
                relation_ordinal,
                created_at,
                updated_at,
            ))
            .into_boxed();

        match order.field {
            WorkRelationField::WorkRelationId => match order.direction {
                Direction::Asc => query = query.order(work_relation_id.asc()),
                Direction::Desc => query = query.order(work_relation_id.desc()),
            },
            WorkRelationField::RelatorWorkId => match order.direction {
                Direction::Asc => query = query.order(relator_work_id.asc()),
                Direction::Desc => query = query.order(relator_work_id.desc()),
            },
            WorkRelationField::RelatedWorkId => match order.direction {
                Direction::Asc => query = query.order(related_work_id.asc()),
                Direction::Desc => query = query.order(related_work_id.desc()),
            },
            WorkRelationField::RelationType => match order.direction {
                Direction::Asc => query = query.order(relation_type.asc()),
                Direction::Desc => query = query.order(relation_type.desc()),
            },
            WorkRelationField::RelationOrdinal => match order.direction {
                Direction::Asc => query = query.order(relation_ordinal.asc()),
                Direction::Desc => query = query.order(relation_ordinal.desc()),
            },
            WorkRelationField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            WorkRelationField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq(any(publishers)));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(relator_work_id.eq(pid));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<WorkRelation>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::work_relation::dsl::*;
        let connection = db.get().unwrap();

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        match work_relation.count().get_result::<i64>(&connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.relator_work_id)?.publisher_id(db)
    }

    crud_methods!(work_relation::table, work_relation::dsl::work_relation);
}

impl HistoryEntry for WorkRelation {
    type NewHistoryEntity = NewWorkRelationHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            work_relation_id: self.work_relation_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewWorkRelationHistory {
    type MainEntity = WorkRelationHistory;

    db_insert!(work_relation_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_relation_pk() {
        let work_relation: WorkRelation = Default::default();
        assert_eq!(work_relation.pk(), work_relation.work_relation_id);
    }

    #[test]
    fn test_new_work_relation_history_from_work_relation() {
        let work_relation: WorkRelation = Default::default();
        let account_id: Uuid = Default::default();
        let new_work_relation_history = work_relation.new_history_entry(&account_id);
        assert_eq!(
            new_work_relation_history.work_relation_id,
            work_relation.work_relation_id
        );
        assert_eq!(new_work_relation_history.account_id, account_id);
        assert_eq!(
            new_work_relation_history.data,
            serde_json::Value::String(serde_json::to_string(&work_relation).unwrap())
        );
    }
}
