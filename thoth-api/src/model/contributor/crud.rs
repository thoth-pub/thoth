use super::{
    Contributor, ContributorField, ContributorHistory, ContributorOrderBy, NewContributor,
    NewContributorHistory, PatchContributor,
};
use crate::db::PgPool;
use crate::model::{Crud, DbInsert, HistoryEntry, PublisherIds};
use crate::schema::{contributor, contributor_history};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Contributor {
    type NewEntity = NewContributor;
    type PatchEntity = PatchContributor;
    type OrderByEntity = ContributorOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.contributor_id
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
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Contributor>> {
        use crate::schema::contributor::dsl::*;
        let mut connection = db.get()?;
        let mut query = contributor.into_boxed();

        query = match order.field {
            ContributorField::ContributorId => {
                apply_directional_order!(query, order.direction, order, contributor_id)
            }
            ContributorField::FirstName => {
                apply_directional_order!(query, order.direction, order, first_name)
            }
            ContributorField::LastName => {
                apply_directional_order!(query, order.direction, order, last_name)
            }
            ContributorField::FullName => {
                apply_directional_order!(query, order.direction, order, full_name)
            }
            ContributorField::Orcid => {
                apply_directional_order!(query, order.direction, order, orcid)
            }
            ContributorField::Website => {
                apply_directional_order!(query, order.direction, order, website)
            }
            ContributorField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            ContributorField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
        };
        if let Some(filter) = filter {
            query = query.filter(
                full_name
                    .ilike(format!("%{filter}%"))
                    .or(last_name.ilike(format!("%{filter}%")))
                    .or(orcid.ilike(format!("%{filter}%"))),
            );
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Contributor>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::contributor::dsl::*;
        let mut connection = db.get()?;
        let mut query = contributor.into_boxed();
        if let Some(filter) = filter {
            query = query.filter(
                full_name
                    .ilike(format!("%{filter}%"))
                    .or(last_name.ilike(format!("%{filter}%")))
                    .or(orcid.ilike(format!("%{filter}%"))),
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
    crud_methods!(contributor::table, contributor::dsl::contributor);
}

impl PublisherIds for Contributor {
    fn publisher_ids(&self, db: &PgPool) -> ThothResult<Vec<Uuid>> {
        let mut connection = db.get()?;
        crate::schema::publisher::table
            .inner_join(crate::schema::imprint::table.inner_join(
                crate::schema::work::table.inner_join(crate::schema::contribution::table),
            ))
            .select(crate::schema::publisher::publisher_id)
            .filter(crate::schema::contribution::contributor_id.eq(self.contributor_id))
            .distinct()
            .load::<Uuid>(&mut connection)
            .map_err(Into::into)
    }
}

impl HistoryEntry for Contributor {
    type NewHistoryEntity = NewContributorHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            contributor_id: self.contributor_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewContributorHistory {
    type MainEntity = ContributorHistory;

    db_insert!(contributor_history::table);
}
