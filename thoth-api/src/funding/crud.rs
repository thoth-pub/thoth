use super::model::{
    Funding, FundingField, FundingHistory, NewFunding, NewFundingHistory, PatchFunding,
};
use crate::errors::{ThothError, ThothResult};
use crate::graphql::model::FundingOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{funding, funding_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

impl Crud for Funding {
    type NewEntity = NewFunding;
    type PatchEntity = PatchFunding;
    type OrderByEntity = FundingOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();

    fn pk(&self) -> uuid::Uuid {
        self.funding_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        _: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<uuid::Uuid>,
        parent_id_1: Option<uuid::Uuid>,
        parent_id_2: Option<uuid::Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Funding>> {
        use crate::schema::funding::dsl::*;
        let connection = db.get().unwrap();
        let mut query = funding
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select((
                funding_id,
                work_id,
                funder_id,
                program,
                project_name,
                project_shortname,
                grant_number,
                jurisdiction,
                created_at,
                updated_at,
            ))
            .into_boxed();

        match order.field {
            FundingField::FundingId => match order.direction {
                Direction::Asc => query = query.order(funding_id.asc()),
                Direction::Desc => query = query.order(funding_id.desc()),
            },
            FundingField::WorkId => match order.direction {
                Direction::Asc => query = query.order(work_id.asc()),
                Direction::Desc => query = query.order(work_id.desc()),
            },
            FundingField::FunderId => match order.direction {
                Direction::Asc => query = query.order(funder_id.asc()),
                Direction::Desc => query = query.order(funder_id.desc()),
            },
            FundingField::Program => match order.direction {
                Direction::Asc => query = query.order(program.asc()),
                Direction::Desc => query = query.order(program.desc()),
            },
            FundingField::ProjectName => match order.direction {
                Direction::Asc => query = query.order(project_name.asc()),
                Direction::Desc => query = query.order(project_name.desc()),
            },
            FundingField::ProjectShortname => match order.direction {
                Direction::Asc => query = query.order(project_shortname.asc()),
                Direction::Desc => query = query.order(project_shortname.desc()),
            },
            FundingField::GrantNumber => match order.direction {
                Direction::Asc => query = query.order(grant_number.asc()),
                Direction::Desc => query = query.order(grant_number.desc()),
            },
            FundingField::Jurisdiction => match order.direction {
                Direction::Asc => query = query.order(jurisdiction.asc()),
                Direction::Desc => query = query.order(jurisdiction.desc()),
            },
            FundingField::CreatedAt => match order.direction {
                Direction::Asc => query = query.order(created_at.asc()),
                Direction::Desc => query = query.order(created_at.desc()),
            },
            FundingField::UpdatedAt => match order.direction {
                Direction::Asc => query = query.order(updated_at.asc()),
                Direction::Desc => query = query.order(updated_at.desc()),
            },
        }
        // This loop must appear before any other filter statements, as it takes advantage of
        // the behaviour of `or_filter` being equal to `filter` when no other filters are present yet.
        // Result needs to be `WHERE (x = $1 [OR x = $2...]) AND ([...])` - note bracketing.
        for pub_id in publishers {
            query = query.or_filter(crate::schema::imprint::publisher_id.eq(pub_id));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(pid) = parent_id_2 {
            query = query.filter(funder_id.eq(pid));
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Funding>(&connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<uuid::Uuid>,
        _: Option<Self::FilterParameter1>,
        _: Option<Self::FilterParameter2>,
    ) -> ThothResult<i32> {
        use crate::schema::funding::dsl::*;
        let connection = db.get().unwrap();
        match funding.count().get_result::<i64>(&connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    crud_methods!(funding::table, funding::dsl::funding);
}

impl HistoryEntry for Funding {
    type NewHistoryEntity = NewFundingHistory;

    fn new_history_entry(&self, account_id: &uuid::Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            funding_id: self.funding_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewFundingHistory {
    type MainEntity = FundingHistory;

    db_insert!(funding_history::table);
}
