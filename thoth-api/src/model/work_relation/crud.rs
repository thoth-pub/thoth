use super::{
    NewWorkRelation, NewWorkRelationHistory, PatchWorkRelation, RelationType, WorkRelation,
    WorkRelationField, WorkRelationHistory, WorkRelationOrderBy,
};
use crate::db_insert;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{work_relation, work_relation_history};
use diesel::dsl::{any, max};
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for WorkRelation {
    type NewEntity = NewWorkRelation;
    type PatchEntity = PatchWorkRelation;
    type OrderByEntity = WorkRelationOrderBy;
    type FilterParameter1 = RelationType;
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
        relation_types: Vec<Self::FilterParameter1>,
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
        if !relation_types.is_empty() {
            query = query.filter(relation_type.eq(any(relation_types)));
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
        relation_types: Vec<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::work_relation::dsl::*;
        let connection = db.get().unwrap();
        let mut query = work_relation.into_boxed();
        if !relation_types.is_empty() {
            query = query.filter(relation_type.eq(any(relation_types)));
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
        crate::model::work::Work::from_id(db, &self.relator_work_id)?.publisher_id(db)
    }

    // `crud_methods!` cannot be used for create(), update() or delete()
    // as we need to execute multiple statements in the same transaction.
    // This function recreates the `crud_methods!` from_id() logic.
    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        let connection = db.get().unwrap();
        match work_relation::table
            .find(entity_id)
            .get_result::<Self>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn create(db: &crate::db::PgPool, data: &NewWorkRelation) -> ThothResult<Self> {
        // For each Relator - Relationship - Related record we create, we must also
        // create the corresponding Related - InverseRelationship - Relator record.
        // Execute all statements within the same transaction,
        // because if one fails, all need to be reverted.
        let connection = db.get().unwrap();
        connection.transaction(|| {
            // We need to determine an appropriate relation_ordinal for the inverse record.
            // Find the current highest ordinal for the relevant work and type.
            // This will return `None` if no records with this work and type already exist.
            let max_inverse_ordinal =
                work_relation::table
                    .select(max(work_relation::relation_ordinal))
                    .filter(work_relation::relator_work_id.eq(data.related_work_id).and(
                        work_relation::relation_type.eq(data.relation_type.convert_to_inverse()),
                    ))
                    .get_result::<Option<i32>>(&connection)
                    .expect("Error loading work relation ordinal values");
            let inverse_data = NewWorkRelation {
                relator_work_id: data.related_work_id,
                related_work_id: data.relator_work_id,
                relation_type: data.relation_type.convert_to_inverse(),
                // Set the ordinal based on the current highest ordinal for this work and type
                // (defaulting to 1 if none exists).
                relation_ordinal: max_inverse_ordinal.unwrap_or_default() + 1,
            };
            diesel::insert_into(work_relation::table)
                .values(&inverse_data)
                .execute(&connection)?;
            match diesel::insert_into(work_relation::table)
                .values(data)
                .get_result::<Self>(&connection)
            {
                Ok(t) => Ok(t),
                Err(e) => Err(ThothError::from(e)),
            }
        })
    }

    fn update(
        &self,
        db: &crate::db::PgPool,
        data: &PatchWorkRelation,
        account_id: &Uuid,
    ) -> ThothResult<Self> {
        // For each Relator - Relationship - Related record we update, we must also
        // update the corresponding Related - InverseRelationship - Relator record.
        // Execute all statements within the same transaction,
        // because if one fails, all need to be reverted.
        let connection = db.get().unwrap();
        connection.transaction(|| {
            // Find the corresponding record. This relies on records being
            // unique across relator work, related work, and relation type.
            let inverse_work_relation = work_relation::table
                .filter(work_relation::relator_work_id.eq(self.related_work_id).and(
                    work_relation::related_work_id.eq(self.relator_work_id).and(
                        work_relation::relation_type.eq(self.relation_type.convert_to_inverse()),
                    ),
                ))
                .first::<WorkRelation>(&connection)
                .expect("Error loading inverse work relation");
            let inverse_data = PatchWorkRelation {
                work_relation_id: inverse_work_relation.work_relation_id,
                relator_work_id: data.related_work_id,
                related_work_id: data.relator_work_id,
                relation_type: data.relation_type.convert_to_inverse(),
                relation_ordinal: inverse_work_relation.relation_ordinal,
            };
            diesel::update(work_relation::table.find(inverse_work_relation.work_relation_id))
                .set(inverse_data)
                .execute(&connection)?;
            match diesel::update(work_relation::table.find(&self.pk()))
                .set(data)
                .get_result::<Self>(&connection)
            {
                // On success, create a new history table entry.
                // Only record the original update, not the automatic inverse update.
                Ok(t) => match self.new_history_entry(account_id).insert(&connection) {
                    Ok(_) => Ok(t),
                    Err(e) => Err(e),
                },
                Err(e) => Err(ThothError::from(e)),
            }
        })
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        // For each Relator - Relationship - Related record we delete, we must also
        // delete the corresponding Related - InverseRelationship - Relator record.
        // Execute all statements within the same transaction,
        // because if one fails, all need to be reverted.
        let connection = db.get().unwrap();
        connection.transaction(|| {
            // Find the corresponding record ID. This relies on records being
            // unique across relator work, related work, and relation type.
            let inverse_work_relation_id = work_relation::table
                .select(work_relation::work_relation_id)
                .filter(work_relation::relator_work_id.eq(self.related_work_id).and(
                    work_relation::related_work_id.eq(self.relator_work_id).and(
                        work_relation::relation_type.eq(self.relation_type.convert_to_inverse()),
                    ),
                ))
                .first::<Uuid>(&connection)
                .expect("Error loading inverse work relation");
            diesel::delete(work_relation::table.find(inverse_work_relation_id))
                .execute(&connection)?;
            match diesel::delete(work_relation::table.find(self.pk())).execute(&connection) {
                Ok(_) => Ok(self),
                Err(e) => Err(ThothError::from(e)),
            }
        })
    }
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
