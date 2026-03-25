use super::{Award, AwardField, AwardHistory, AwardOrderBy, NewAward, NewAwardHistory, PatchAward};
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{award, award_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

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
        let mut connection = db.get()?;
        let mut query = award
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::award::all_columns)
            .into_boxed();

        query = match order.field {
            AwardField::AwardId => {
                apply_directional_order!(query, order.direction, order, award_id)
            }
            AwardField::WorkId => apply_directional_order!(query, order.direction, order, work_id),
            AwardField::AwardOrdinal => {
                apply_directional_order!(query, order.direction, order, award_ordinal)
            }
            AwardField::Title => apply_directional_order!(query, order.direction, order, title),
            AwardField::Category => {
                apply_directional_order!(query, order.direction, order, category)
            }
            AwardField::Year => apply_directional_order!(query, order.direction, order, year),
            AwardField::Jury => apply_directional_order!(query, order.direction, order, jury),
            AwardField::Country => {
                apply_directional_order!(query, order.direction, order, country)
            }
            AwardField::Role => apply_directional_order!(query, order.direction, order, role),
            AwardField::Url => apply_directional_order!(query, order.direction, order, url),
            AwardField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            AwardField::UpdatedAt => {
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
                    title
                        .ilike(format!("%{filter}%"))
                        .or(category.ilike(format!("%{filter}%")))
                        .or(year.ilike(format!("%{filter}%")))
                        .or(jury.ilike(format!("%{filter}%")))
                        .or(prize_statement.ilike(format!("%{filter}%")))
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
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    title
                        .ilike(format!("%{filter}%"))
                        .or(category.ilike(format!("%{filter}%")))
                        .or(year.ilike(format!("%{filter}%")))
                        .or(jury.ilike(format!("%{filter}%")))
                        .or(prize_statement.ilike(format!("%{filter}%")))
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

    crud_methods!(award::table, award::dsl::award);
}

publisher_id_impls!(Award, NewAward, PatchAward, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for Award {
    type NewHistoryEntity = NewAwardHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            award_id: self.award_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewAwardHistory {
    type MainEntity = AwardHistory;

    db_insert!(award_history::table);
}

impl Reorder for Award {
    db_change_ordinal!(
        award::table,
        award::award_ordinal,
        "award_award_ordinal_work_id_uniq"
    );

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
