use super::{
    Imprint, ImprintField, ImprintHistory, ImprintOrderBy, NewImprint, NewImprintHistory,
    PatchImprint,
};
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{imprint, imprint_history};
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
    type FilterParameter3 = ();
    type FilterParameter4 = ();

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
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Imprint>> {
        use crate::schema::imprint::dsl::*;
        let mut connection = db.get()?;
        let mut query = imprint.into_boxed();

        query = match order.field {
            ImprintField::ImprintId => {
                apply_directional_order!(query, order.direction, order, imprint_id)
            }
            ImprintField::ImprintName => {
                apply_directional_order!(query, order.direction, order, imprint_name)
            }
            ImprintField::ImprintUrl => {
                apply_directional_order!(query, order.direction, order, imprint_url)
            }
            ImprintField::CrossmarkDoi => {
                apply_directional_order!(query, order.direction, order, crossmark_doi)
            }
            ImprintField::DefaultCurrency => {
                apply_directional_order!(query, order.direction, order, default_currency)
            }
            ImprintField::DefaultPlace => {
                apply_directional_order!(query, order.direction, order, default_place)
            }
            ImprintField::DefaultLocale => {
                apply_directional_order!(query, order.direction, order, default_locale)
            }
            ImprintField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            ImprintField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
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
                    .ilike(format!("%{filter}%"))
                    .or(imprint_url.ilike(format!("%{filter}%"))),
            );
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Imprint>(&mut connection)
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
        use crate::schema::imprint::dsl::*;
        let mut connection = db.get()?;
        let mut query = imprint.into_boxed();
        if !publishers.is_empty() {
            query = query.filter(publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            query = query.filter(
                imprint_name
                    .ilike(format!("%{filter}%"))
                    .or(imprint_url.ilike(format!("%{filter}%"))),
            );
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

    crud_methods!(imprint::table, imprint::dsl::imprint, without_delete);

    /// Deletes the Imprint through BE-06's publisher-set-first deletion unit
    /// (R52B section 13.4), which retires its Works' actionable work-level jobs
    /// first.
    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        work_upsert_delete_imprint(db, self.imprint_id).map(|_| self)
    }
}

/// `deleteImprint` (R52B section 13.4): the Work unit's shape over the
/// imprint's Works, with the imprint row locked `FOR UPDATE` and its Works
/// locked ascending before retirement.
fn work_upsert_delete_imprint(db: &crate::db::PgPool, imprint_id: Uuid) -> ThothResult<()> {
    use crate::model::work::crud::{
        lock_publisher_set, retire_actionable_work_upsert_jobs, run_work_upsert_deletion_unit,
        within_locked_set, work_upsert_bound_publishers,
    };
    use crate::model::work_upsert::WorkUpsertQueryResultExt;
    use crate::schema::work;
    use diesel::OptionalExtension;
    run_work_upsert_deletion_unit(db, |connection| {
        let works = work::table
            .filter(work::imprint_id.eq(imprint_id))
            .select(work::work_id)
            .load::<Uuid>(connection)
            .work_upsert()?;
        let current = imprint::table
            .filter(imprint::imprint_id.eq(imprint_id))
            .select(imprint::publisher_id)
            .first::<Uuid>(connection)
            .optional()
            .work_upsert()?;
        let Some(current) = current else {
            return Ok(());
        };
        let mut locked = work_upsert_bound_publishers(connection, &works)?;
        locked.push(current);
        locked.sort();
        locked.dedup();
        lock_publisher_set(connection, &locked, None)?;
        let held = imprint::table
            .filter(imprint::imprint_id.eq(imprint_id))
            .select(imprint::publisher_id)
            .for_update()
            .first::<Uuid>(connection)
            .optional()
            .work_upsert()?;
        let Some(held_publisher) = held else {
            return Ok(());
        };
        let works = work::table
            .filter(work::imprint_id.eq(imprint_id))
            .select(work::work_id)
            .order(work::work_id.asc())
            .for_update()
            .load::<Uuid>(connection)
            .work_upsert()?;
        let mut now = work_upsert_bound_publishers(connection, &works)?;
        now.push(held_publisher);
        if !within_locked_set(&locked, &now) {
            return Err(ThothError::WorkDeleteBindingDrift);
        }
        retire_actionable_work_upsert_jobs(connection, &works)?;
        diesel::delete(imprint::table.find(imprint_id))
            .execute(connection)
            .map(|_| ())
            .map_err(Into::into)
    })
}

publisher_id_impls!(Imprint, NewImprint, PatchImprint, |s, _db| {
    Ok(s.publisher_id)
});

impl HistoryEntry for Imprint {
    type NewHistoryEntity = NewImprintHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            imprint_id: self.imprint_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewImprintHistory {
    type MainEntity = ImprintHistory;

    db_insert!(imprint_history::table);
}
