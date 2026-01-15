use super::{Award, AwardField, AwardOrderBy, NewAward, NewAwardHistory, PatchAward};
use crate::graphql::utils::Direction;
use crate::model::work::WorkType;
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::imprint;
use crate::schema::{award, award::dsl, award_history, work};
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
            "Awards can only be attached to book records, not chapters".to_string(),
        ));
    }
    Ok(())
}

impl Crud for Award {
    type NewEntity = NewAward;
    type PatchEntity = PatchAward;
    type OrderByEntity = AwardOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.award_id
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
    ) -> ThothResult<Vec<Award>> {
        use crate::schema::award::dsl::*;
        use crate::schema::imprint;
        let mut connection = db.get()?;
        let mut query = award
            .inner_join(work::table.inner_join(imprint::table))
            .select(crate::schema::award::all_columns)
            .into_boxed();

        query = match order.field {
            AwardField::AwardId => match order.direction {
                Direction::Asc => query.order(award_id.asc()),
                Direction::Desc => query.order(award_id.desc()),
            },
            AwardField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            AwardField::AwardOrdinal => match order.direction {
                Direction::Asc => query.order(award_ordinal.asc()),
                Direction::Desc => query.order(award_ordinal.desc()),
            },
            AwardField::Title => match order.direction {
                Direction::Asc => query.order(title.asc()),
                Direction::Desc => query.order(title.desc()),
            },
            AwardField::Category => match order.direction {
                Direction::Asc => query.order(category.asc()),
                Direction::Desc => query.order(category.desc()),
            },
            AwardField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            AwardField::UpdatedAt => match order.direction {
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
                    title
                        .ilike(format!("%{filter}%"))
                        .or(category.ilike(format!("%{filter}%")))
                        .or(note.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Award>(&mut connection)
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
        use crate::schema::award::dsl::*;
        let mut connection = db.get()?;
        let mut query = award
            .inner_join(work::table.inner_join(imprint::table))
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    title
                        .ilike(format!("%{filter}%"))
                        .or(category.ilike(format!("%{filter}%")))
                        .or(note.ilike(format!("%{filter}%")))
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
        diesel::insert_into(award::table)
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
            diesel::update(dsl::award.find(&self.pk()))
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
        let mut connection = db.get()?;
        dsl::award
            .find(entity_id)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::delete(dsl::award.find(&self.pk()))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(Into::into)
    }
}

impl Reorder for Award {
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

            diesel::sql_query("SET CONSTRAINTS idx_award_workid_ordinal DEFERRED")
                .execute(connection)?;
            for (id, ordinal) in other_objects {
                if new_ordinal > current_ordinal {
                    if ordinal > current_ordinal && ordinal <= new_ordinal {
                        let updated_ordinal = ordinal - 1;
                        diesel::update(award::table.find(id))
                            .set(award::award_ordinal.eq(&updated_ordinal))
                            .execute(connection)?;
                    }
                } else if ordinal >= new_ordinal && ordinal < current_ordinal {
                    let updated_ordinal = ordinal + 1;
                    diesel::update(award::table.find(id))
                        .set(award::award_ordinal.eq(&updated_ordinal))
                        .execute(connection)?;
                }
            }
            diesel::update(award::table.find(self.award_id))
                .set(award::award_ordinal.eq(&new_ordinal))
                .get_result::<Self>(connection)
                .map_err(Into::into)
        })
    }

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        award::table
            .select((award::award_id, award::award_ordinal))
            .filter(
                award::work_id
                    .eq(self.work_id)
                    .and(award::award_id.ne(self.award_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}

impl HistoryEntry for Award {
    type NewHistoryEntity = NewAwardHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            award_id: self.award_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewAwardHistory {
    type MainEntity = super::AwardHistory;

    db_insert!(award_history::table);
}
