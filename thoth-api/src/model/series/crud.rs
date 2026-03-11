use super::{
    NewSeries, NewSeriesHistory, PatchSeries, Series, SeriesField, SeriesHistory, SeriesOrderBy,
    SeriesType,
};
use crate::model::{Crud, DbInsert, HistoryEntry, PublisherId};
use crate::schema::{series, series_history};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Series {
    type NewEntity = NewSeries;
    type PatchEntity = PatchSeries;
    type OrderByEntity = SeriesOrderBy;
    type FilterParameter1 = SeriesType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.series_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        series_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Series>> {
        use crate::schema::series::dsl::*;
        let mut connection = db.get()?;
        let mut query = series
            .inner_join(crate::schema::imprint::table)
            .select(crate::schema::series::all_columns)
            .into_boxed();

        query = match order.field {
            SeriesField::SeriesId => {
                apply_directional_order!(query, order.direction, order, series_id)
            }
            SeriesField::SeriesType => {
                apply_directional_order!(query, order.direction, order, series_type)
            }
            SeriesField::SeriesName => {
                apply_directional_order!(query, order.direction, order, series_name)
            }
            SeriesField::IssnPrint => {
                apply_directional_order!(query, order.direction, order, issn_print)
            }
            SeriesField::IssnDigital => {
                apply_directional_order!(query, order.direction, order, issn_digital)
            }
            SeriesField::SeriesUrl => {
                apply_directional_order!(query, order.direction, order, series_url)
            }
            SeriesField::SeriesDescription => {
                apply_directional_order!(query, order.direction, order, series_description)
            }
            SeriesField::SeriesCfpUrl => {
                apply_directional_order!(query, order.direction, order, series_cfp_url)
            }
            SeriesField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            SeriesField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if !series_types.is_empty() {
            query = query.filter(series_type.eq_any(series_types));
        }
        if let Some(filter) = filter {
            query = query.filter(
                series_name
                    .ilike(format!("%{filter}%"))
                    .or(issn_print.ilike(format!("%{filter}%")))
                    .or(issn_digital.ilike(format!("%{filter}%")))
                    .or(series_url.ilike(format!("%{filter}%")))
                    .or(series_description.ilike(format!("%{filter}%"))),
            );
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Series>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        series_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::series::dsl::*;
        let mut connection = db.get()?;
        let mut query = series
            .inner_join(crate::schema::imprint::table)
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if !series_types.is_empty() {
            query = query.filter(series_type.eq_any(series_types));
        }
        if let Some(filter) = filter {
            query = query.filter(
                series_name
                    .ilike(format!("%{filter}%"))
                    .or(issn_print.ilike(format!("%{filter}%")))
                    .or(issn_digital.ilike(format!("%{filter}%")))
                    .or(series_url.ilike(format!("%{filter}%")))
                    .or(series_description.ilike(format!("%{filter}%"))),
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

    crud_methods!(series::table, series::dsl::series);
}

publisher_id_impls!(Series, NewSeries, PatchSeries, |s, db| {
    let imprint = crate::model::imprint::Imprint::from_id(db, &s.imprint_id)?;
    <crate::model::imprint::Imprint as PublisherId>::publisher_id(&imprint, db)
});

impl HistoryEntry for Series {
    type NewHistoryEntity = NewSeriesHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            series_id: self.series_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewSeriesHistory {
    type MainEntity = SeriesHistory;

    db_insert!(series_history::table);
}
