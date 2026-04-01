use super::{
    Endorsement, EndorsementField, EndorsementHistory, EndorsementOrderBy, NewEndorsement,
    NewEndorsementHistory, PatchEndorsement,
};
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{endorsement, endorsement_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

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
        let mut connection = db.get()?;
        let mut query = endorsement
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::endorsement::all_columns)
            .into_boxed();

        query = match order.field {
            EndorsementField::EndorsementId => {
                apply_directional_order!(query, order.direction, order, endorsement_id)
            }
            EndorsementField::WorkId => {
                apply_directional_order!(query, order.direction, order, work_id)
            }
            EndorsementField::EndorsementOrdinal => {
                apply_directional_order!(query, order.direction, order, endorsement_ordinal)
            }
            EndorsementField::AuthorName => {
                apply_directional_order!(query, order.direction, order, author_name)
            }
            EndorsementField::AuthorRole => {
                apply_directional_order!(query, order.direction, order, author_role)
            }
            EndorsementField::Url => apply_directional_order!(query, order.direction, order, url),
            EndorsementField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            EndorsementField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
        };

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
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
                        .or(author_orcid.ilike(format!("%{filter}%")))
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
        let mut connection = db.get()?;
        let mut query = endorsement
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    author_name
                        .ilike(format!("%{filter}%"))
                        .or(author_role.ilike(format!("%{filter}%")))
                        .or(author_orcid.ilike(format!("%{filter}%")))
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

    crud_methods!(endorsement::table, endorsement::dsl::endorsement);
}

publisher_id_impls!(Endorsement, NewEndorsement, PatchEndorsement, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for Endorsement {
    type NewHistoryEntity = NewEndorsementHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            endorsement_id: self.endorsement_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewEndorsementHistory {
    type MainEntity = EndorsementHistory;

    db_insert!(endorsement_history::table);
}

impl Reorder for Endorsement {
    db_change_ordinal!(
        endorsement::table,
        endorsement::endorsement_ordinal,
        "endorsement_endorsement_ordinal_work_id_uniq"
    );

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
