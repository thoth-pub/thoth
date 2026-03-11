use super::{
    AdditionalResource, AdditionalResourceField, AdditionalResourceHistory,
    AdditionalResourceOrderBy, NewAdditionalResource, NewAdditionalResourceHistory,
    PatchAdditionalResource,
};
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{additional_resource, additional_resource_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for AdditionalResource {
    type NewEntity = NewAdditionalResource;
    type PatchEntity = PatchAdditionalResource;
    type OrderByEntity = AdditionalResourceOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.additional_resource_id
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
    ) -> ThothResult<Vec<AdditionalResource>> {
        use crate::schema::additional_resource::dsl::*;
        let mut connection = db.get()?;
        let mut query = additional_resource
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::additional_resource::all_columns)
            .into_boxed();

        query = match order.field {
            AdditionalResourceField::AdditionalResourceId => {
                apply_directional_order!(query, order.direction, order, additional_resource_id)
            }
            AdditionalResourceField::WorkId => {
                apply_directional_order!(query, order.direction, order, work_id)
            }
            AdditionalResourceField::ResourceOrdinal => {
                apply_directional_order!(query, order.direction, order, resource_ordinal)
            }
            AdditionalResourceField::Title => {
                apply_directional_order!(query, order.direction, order, title)
            }
            AdditionalResourceField::Attribution => {
                apply_directional_order!(query, order.direction, order, attribution)
            }
            AdditionalResourceField::ResourceType => {
                apply_directional_order!(query, order.direction, order, resource_type)
            }
            AdditionalResourceField::Doi => {
                apply_directional_order!(query, order.direction, order, doi)
            }
            AdditionalResourceField::Handle => {
                apply_directional_order!(query, order.direction, order, handle)
            }
            AdditionalResourceField::Url => {
                apply_directional_order!(query, order.direction, order, url)
            }
            AdditionalResourceField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            AdditionalResourceField::UpdatedAt => {
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
                        .or(description.ilike(format!("%{filter}%")))
                        .or(attribution.ilike(format!("%{filter}%")))
                        .or(doi.ilike(format!("%{filter}%")))
                        .or(handle.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<AdditionalResource>(&mut connection)
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
        use crate::schema::additional_resource::dsl::*;
        let mut connection = db.get()?;
        let mut query = additional_resource
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
                        .or(description.ilike(format!("%{filter}%")))
                        .or(attribution.ilike(format!("%{filter}%")))
                        .or(doi.ilike(format!("%{filter}%")))
                        .or(handle.ilike(format!("%{filter}%")))
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

    crud_methods!(
        additional_resource::table,
        additional_resource::dsl::additional_resource
    );
}

publisher_id_impls!(
    AdditionalResource,
    NewAdditionalResource,
    PatchAdditionalResource,
    |s, db| { crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db) }
);

impl HistoryEntry for AdditionalResource {
    type NewHistoryEntity = NewAdditionalResourceHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            additional_resource_id: self.additional_resource_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewAdditionalResourceHistory {
    type MainEntity = AdditionalResourceHistory;

    db_insert!(additional_resource_history::table);
}

impl Reorder for AdditionalResource {
    db_change_ordinal!(
        additional_resource::table,
        additional_resource::resource_ordinal,
        "additional_resource_resource_ordinal_work_id_uniq"
    );

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        additional_resource::table
            .select((
                additional_resource::additional_resource_id,
                additional_resource::resource_ordinal,
            ))
            .filter(
                additional_resource::work_id.eq(self.work_id).and(
                    additional_resource::additional_resource_id.ne(self.additional_resource_id),
                ),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}
