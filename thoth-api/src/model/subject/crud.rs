use super::{
    NewSubject, NewSubjectHistory, PatchSubject, Subject, SubjectField, SubjectHistory, SubjectType,
};
use crate::graphql::model::SubjectOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{subject, subject_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Subject {
    type NewEntity = NewSubject;
    type PatchEntity = PatchSubject;
    type OrderByEntity = SubjectOrderBy;
    type FilterParameter1 = SubjectType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();

    fn pk(&self) -> Uuid {
        self.subject_id
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
        subject_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Subject>> {
        use crate::schema::subject::dsl::*;
        let mut connection = db.get()?;
        let mut query = subject
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::subject::all_columns)
            .into_boxed();

        query = match order.field {
            SubjectField::SubjectId => match order.direction {
                Direction::Asc => query.order(subject_id.asc()),
                Direction::Desc => query.order(subject_id.desc()),
            },
            SubjectField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            SubjectField::SubjectType => match order.direction {
                Direction::Asc => query.order(subject_type.asc()),
                Direction::Desc => query.order(subject_type.desc()),
            },
            SubjectField::SubjectCode => match order.direction {
                Direction::Asc => query.order(subject_code.asc()),
                Direction::Desc => query.order(subject_code.desc()),
            },
            SubjectField::SubjectOrdinal => match order.direction {
                Direction::Asc => query.order(subject_ordinal.asc()),
                Direction::Desc => query.order(subject_ordinal.desc()),
            },
            SubjectField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            SubjectField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if !subject_types.is_empty() {
            query = query.filter(subject_type.eq_any(subject_types));
        }
        if let Some(filter) = filter {
            query = query.filter(subject_code.ilike(format!("%{filter}%")));
        }
        query
            .then_order_by(subject_code.asc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Subject>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        subject_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::subject::dsl::*;
        let mut connection = db.get()?;
        let mut query = subject.into_boxed();
        if !subject_types.is_empty() {
            query = query.filter(subject_type.eq_any(subject_types));
        }
        if let Some(filter) = filter {
            query = query.filter(subject_code.ilike(format!("%{filter}%")));
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

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    crud_methods!(subject::table, subject::dsl::subject);
}

impl HistoryEntry for Subject {
    type NewHistoryEntity = NewSubjectHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            subject_id: self.subject_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewSubjectHistory {
    type MainEntity = SubjectHistory;

    db_insert!(subject_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_pk() {
        let subject: Subject = Default::default();
        assert_eq!(subject.pk(), subject.subject_id);
    }

    #[test]
    fn test_new_subject_history_from_subject() {
        let subject: Subject = Default::default();
        let user_id = "1234567".to_string();
        let new_subject_history = subject.new_history_entry(&user_id);
        assert_eq!(new_subject_history.subject_id, subject.subject_id);
        assert_eq!(new_subject_history.user_id, user_id);
        assert_eq!(
            new_subject_history.data,
            serde_json::Value::String(serde_json::to_string(&subject).unwrap())
        );
    }
}
