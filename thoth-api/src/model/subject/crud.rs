use super::{
    NewSubject, NewSubjectHistory, PatchSubject, Subject, SubjectField, SubjectHistory, SubjectType,
};
use crate::graphql::types::inputs::SubjectOrderBy;
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{subject, subject_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Subject {
    type NewEntity = NewSubject;
    type PatchEntity = PatchSubject;
    type OrderByEntity = SubjectOrderBy;
    type FilterParameter1 = SubjectType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

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
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Subject>> {
        use crate::schema::subject::dsl::*;
        let mut connection = db.get()?;
        let mut query = subject
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::subject::all_columns)
            .into_boxed();

        query = match order.field {
            SubjectField::SubjectId => {
                apply_directional_order!(query, order.direction, order, subject_id)
            }
            SubjectField::WorkId => {
                apply_directional_order!(query, order.direction, order, work_id)
            }
            SubjectField::SubjectType => {
                apply_directional_order!(query, order.direction, order, subject_type)
            }
            SubjectField::SubjectCode => {
                apply_directional_order!(query, order.direction, order, subject_code)
            }
            SubjectField::SubjectOrdinal => {
                apply_directional_order!(query, order.direction, order, subject_ordinal)
            }
            SubjectField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            SubjectField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
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
        _: Option<Self::FilterParameter4>,
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

    crud_methods!(subject::table, subject::dsl::subject);
}

publisher_id_impls!(Subject, NewSubject, PatchSubject, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

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

impl Reorder for Subject {
    db_change_ordinal!(
        subject::table,
        subject::subject_ordinal,
        "subject_ordinal_type_uniq"
    );

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        subject::table
            .select((subject::subject_id, subject::subject_ordinal))
            .filter(
                subject::work_id
                    .eq(self.work_id)
                    .and(subject::subject_type.eq(self.subject_type))
                    .and(subject::subject_id.ne(self.subject_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}
