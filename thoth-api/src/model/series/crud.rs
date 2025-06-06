use super::{
    NewSeries, NewSeriesHistory, PatchSeries, Series, SeriesField, SeriesHistory, SeriesOrderBy,
    SeriesType,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{series, series_history};
use crate::{crud_methods, db_insert};
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
    ) -> ThothResult<Vec<Series>> {
        use crate::schema::series::dsl::*;
        let mut connection = db.get()?;
        let mut query = series
            .inner_join(crate::schema::imprint::table)
            .select(crate::schema::series::all_columns)
            .into_boxed();

        query = match order.field {
            SeriesField::SeriesId => match order.direction {
                Direction::Asc => query.order(series_id.asc()),
                Direction::Desc => query.order(series_id.desc()),
            },
            SeriesField::SeriesType => match order.direction {
                Direction::Asc => query.order(series_type.asc()),
                Direction::Desc => query.order(series_type.desc()),
            },
            SeriesField::SeriesName => match order.direction {
                Direction::Asc => query.order(series_name.asc()),
                Direction::Desc => query.order(series_name.desc()),
            },
            SeriesField::IssnPrint => match order.direction {
                Direction::Asc => query.order(issn_print.asc()),
                Direction::Desc => query.order(issn_print.desc()),
            },
            SeriesField::IssnDigital => match order.direction {
                Direction::Asc => query.order(issn_digital.asc()),
                Direction::Desc => query.order(issn_digital.desc()),
            },
            SeriesField::SeriesUrl => match order.direction {
                Direction::Asc => query.order(series_url.asc()),
                Direction::Desc => query.order(series_url.desc()),
            },
            SeriesField::SeriesDescription => match order.direction {
                Direction::Asc => query.order(series_description.asc()),
                Direction::Desc => query.order(series_description.desc()),
            },
            SeriesField::SeriesCfpUrl => match order.direction {
                Direction::Asc => query.order(series_cfp_url.asc()),
                Direction::Desc => query.order(series_cfp_url.desc()),
            },
            SeriesField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            SeriesField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
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

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        let imprint = crate::model::imprint::Imprint::from_id(db, &self.imprint_id)?;
        <crate::model::imprint::Imprint as Crud>::publisher_id(&imprint, db)
    }

    crud_methods!(series::table, series::dsl::series);
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_pk() {
        let series: Series = Default::default();
        assert_eq!(series.pk(), series.series_id);
    }

    #[test]
    fn test_new_series_history_from_series() {
        let series: Series = Default::default();
        let user_id = "123456".to_string();
        let new_series_history = series.new_history_entry(&user_id);
        assert_eq!(new_series_history.series_id, series.series_id);
        assert_eq!(new_series_history.user_id, user_id);
        assert_eq!(
            new_series_history.data,
            serde_json::Value::String(serde_json::to_string(&series).unwrap())
        );
    }
}
