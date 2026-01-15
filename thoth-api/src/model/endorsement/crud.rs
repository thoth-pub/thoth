use super::{
    Endorsement, EndorsementField, EndorsementOrderBy, NewEndorsement, NewEndorsementHistory,
    PatchEndorsement,
};
use crate::graphql::utils::Direction;
use crate::model::work::WorkType;
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{endorsement, endorsement::dsl, endorsement_history, work};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

fn validate_book_only(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<()> {
    let mut connection = db.get()?;
    let work_type = work::table
        .select(work::work_type)
        .filter(work::work_id.eq(work_id))
        .first::<WorkType>(&mut connection)?;

    if work_type == WorkType::BookChapter {
        return Err(ThothError::InternalError(
            "Endorsements can only be attached to book records, not chapters".to_string(),
        ));
    }
    Ok(())
}

impl Crud for Endorsement {
    type NewEntity = NewEndorsement;
    type PatchEntity = PatchEndorsement;
    type OrderByEntity = EndorsementOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.endorsement_id
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
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Endorsement>> {
        use crate::schema::endorsement::dsl::*;
        use crate::schema::imprint;
        let mut connection = db.get()?;
        let mut query = endorsement
            .inner_join(work::table.inner_join(imprint::table))
            .select(crate::schema::endorsement::all_columns)
            .into_boxed();

        query = match order.field {
            EndorsementField::EndorsementId => match order.direction {
                Direction::Asc => query.order(endorsement_id.asc()),
                Direction::Desc => query.order(endorsement_id.desc()),
            },
            EndorsementField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            EndorsementField::EndorsementOrdinal => match order.direction {
                Direction::Asc => query.order(endorsement_ordinal.asc()),
                Direction::Desc => query.order(endorsement_ordinal.desc()),
            },
            EndorsementField::AuthorName => match order.direction {
                Direction::Asc => query.order(author_name.asc()),
                Direction::Desc => query.order(author_name.desc()),
            },
            EndorsementField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            EndorsementField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };

        if !publishers.is_empty() {
            query = query.filter(imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    author_name
                        .ilike(format!("%{filter}%"))
                        .or(author_role.ilike(format!("%{filter}%")))
                        .or(text.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Endorsement>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::endorsement::dsl::*;
        use crate::schema::imprint;
        let mut connection = db.get()?;
        let mut query = endorsement
            .inner_join(work::table.inner_join(imprint::table))
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    author_name
                        .ilike(format!("%{filter}%"))
                        .or(author_role.ilike(format!("%{filter}%")))
                        .or(text.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }

        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> ThothResult<Self> {
        validate_book_only(db, &data.work_id)?;
        let mut connection = db.get()?;
        diesel::insert_into(endorsement::table)
            .values(data)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn update(
        &self,
        db: &crate::db::PgPool,
        data: &Self::PatchEntity,
        account_id: &Uuid,
    ) -> ThothResult<Self> {
        validate_book_only(db, &self.work_id)?;
        let mut connection = db.get()?;
        connection.transaction(|connection| {
            diesel::update(dsl::endorsement.find(&self.pk()))
                .set(data)
                .get_result(connection)
                .map_err(Into::into)
                .and_then(|c| {
                    self.new_history_entry(account_id)
                        .insert(connection)
                        .map(|_| c)
                })
        })
    }

    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = db.get()?;
        dsl::endorsement
            .find(entity_id)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = db.get()?;
        diesel::delete(dsl::endorsement.find(&self.pk()))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(Into::into)
    }
}

impl Reorder for Endorsement {
    fn change_ordinal(
        &self,
        db: &crate::db::PgPool,
        current_ordinal: i32,
        new_ordinal: i32,
        _account_id: &Uuid,
    ) -> ThothResult<Self> {
        let mut connection = db.get()?;
        connection.transaction(|connection| {
            if current_ordinal == new_ordinal {
                return Ok(self.clone());
            }

            let mut other_objects = self.get_other_objects(connection)?;
            other_objects.sort_by_key(|(_, ordinal)| *ordinal);

            diesel::sql_query("SET CONSTRAINTS idx_endorsement_workid_ordinal DEFERRED")
                .execute(connection)?;
            for (id, ordinal) in other_objects {
                if new_ordinal > current_ordinal {
                    if ordinal > current_ordinal && ordinal <= new_ordinal {
                        let updated_ordinal = ordinal - 1;
                        diesel::update(endorsement::table.find(id))
                            .set(endorsement::endorsement_ordinal.eq(&updated_ordinal))
                            .execute(connection)?;
                    }
                } else if ordinal >= new_ordinal && ordinal < current_ordinal {
                    let updated_ordinal = ordinal + 1;
                    diesel::update(endorsement::table.find(id))
                        .set(endorsement::endorsement_ordinal.eq(&updated_ordinal))
                        .execute(connection)?;
                }
            }
            diesel::update(endorsement::table.find(self.endorsement_id))
                .set(endorsement::endorsement_ordinal.eq(&new_ordinal))
                .get_result::<Self>(connection)
                .map_err(Into::into)
        })
    }

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        endorsement::table
            .select((
                endorsement::endorsement_id,
                endorsement::endorsement_ordinal,
            ))
            .filter(
                endorsement::work_id
                    .eq(self.work_id)
                    .and(endorsement::endorsement_id.ne(self.endorsement_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}

impl HistoryEntry for Endorsement {
    type NewHistoryEntity = NewEndorsementHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            endorsement_id: self.endorsement_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewEndorsementHistory {
    type MainEntity = super::EndorsementHistory;

    db_insert!(endorsement_history::table);
}
