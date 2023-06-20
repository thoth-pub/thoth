use super::{
    NewSeries, NewSeriesHistory, PatchSeries, Series, SeriesField, SeriesFilter, SeriesHistory,
    SeriesOrderBy, SeriesType,
};
use crate::graphql::utils::{Direction, Operator};
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{series, series_history};
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, BoxableExpression, NullableExpressionMethods,
};
use diesel::pg::Pg;
use diesel::sql_types::{Bool, Nullable};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Series {
    type NewEntity = NewSeries;
    type PatchEntity = PatchSeries;
    type OrderByEntity = SeriesOrderBy;
    type FilterParameter1 = SeriesType;
    type FilterParameter2 = ();
    type FilterParameter3 = Vec<SeriesFilter>;

    fn pk(&self) -> Uuid {
        self.series_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        _: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        filters: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Series>> {
        use crate::schema::series::dsl::*;
        let mut connection = db.get().unwrap();
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
        let mut filters_unwrapped = filters.unwrap();
        if !filters_unwrapped.is_empty() {
            filters_unwrapped.sort_by_key(|f| f.value.clone());
            let mut prev_value = None;
            // Start with a trivially true statement that we can build on
            // From the docs: "[`.nullable()`] has no impact on the generated SQL, and is
            // only used to allow certain comparisons that would otherwise fail to compile."
            let mut filter_by: Box<dyn BoxableExpression<series, Pg, SqlType = Nullable<Bool>>> = Box::new(series_id.eq(series_id).nullable());
            for filter in filters_unwrapped {
                let curr_value = filter.value.clone();
                if prev_value == Some(curr_value.clone()) {
                    filter_by = match filter.field {
                        // Filtering only supported for text fields
                        SeriesField::SeriesId => filter_by,
                        SeriesField::SeriesType => filter_by,
                        SeriesField::SeriesName => match filter.operator {
                            Operator::Eq => Box::new(filter_by.or(series_name.eq(filter.value).nullable())),
                            Operator::Neq => Box::new(filter_by.or(series_name.ne(filter.value).nullable())),
                            Operator::Gt => Box::new(filter_by.or(series_name.gt(filter.value).nullable())),
                            Operator::Lt => Box::new(filter_by.or(series_name.lt(filter.value).nullable())),
                            Operator::Gte => Box::new(filter_by.or(series_name.ge(filter.value).nullable())),
                            Operator::Lte => Box::new(filter_by.or(series_name.le(filter.value).nullable())),
                            Operator::Ilike => {
                                Box::new(filter_by.or(series_name.ilike(format!("%{}%", filter.value)).nullable()))
                            }
                        },
                        SeriesField::IssnPrint => match filter.operator {
                            Operator::Eq => Box::new(filter_by.or(issn_print.eq(filter.value).nullable())),
                            Operator::Neq => Box::new(filter_by.or(issn_print.ne(filter.value).nullable())),
                            Operator::Gt => Box::new(filter_by.or(issn_print.gt(filter.value).nullable())),
                            Operator::Lt => Box::new(filter_by.or(issn_print.lt(filter.value).nullable())),
                            Operator::Gte => Box::new(filter_by.or(issn_print.ge(filter.value).nullable())),
                            Operator::Lte => Box::new(filter_by.or(issn_print.le(filter.value).nullable())),
                            Operator::Ilike => {
                                Box::new(filter_by.or(issn_print.ilike(format!("%{}%", filter.value)).nullable()))
                            }
                        },
                        SeriesField::IssnDigital => match filter.operator {
                            Operator::Eq => Box::new(filter_by.or(issn_digital.eq(filter.value).nullable())),
                            Operator::Neq => Box::new(filter_by.or(issn_digital.ne(filter.value).nullable())),
                            Operator::Gt => Box::new(filter_by.or(issn_digital.gt(filter.value).nullable())),
                            Operator::Lt => Box::new(filter_by.or(issn_digital.lt(filter.value).nullable())),
                            Operator::Gte => Box::new(filter_by.or(issn_digital.ge(filter.value).nullable())),
                            Operator::Lte => Box::new(filter_by.or(issn_digital.le(filter.value).nullable())),
                            Operator::Ilike => {
                                Box::new(filter_by.or(issn_digital.ilike(format!("%{}%", filter.value)).nullable()))
                            }
                        },
                        SeriesField::SeriesUrl => match filter.operator {
                            Operator::Eq => Box::new(filter_by.or(series_url.eq(filter.value))),
                            Operator::Neq => Box::new(filter_by.or(series_url.ne(filter.value))),
                            Operator::Gt => Box::new(filter_by.or(series_url.gt(filter.value))),
                            Operator::Lt => Box::new(filter_by.or(series_url.lt(filter.value))),
                            Operator::Gte => Box::new(filter_by.or(series_url.ge(filter.value))),
                            Operator::Lte => Box::new(filter_by.or(series_url.le(filter.value))),
                            Operator::Ilike => {
                                Box::new(filter_by.or(series_url.ilike(format!("%{}%", filter.value))))
                            }
                        },
                        SeriesField::SeriesDescription => match filter.operator {
                            Operator::Eq => Box::new(filter_by.or(series_description.eq(filter.value))),
                            Operator::Neq => Box::new(filter_by.or(series_description.ne(filter.value))),
                            Operator::Gt => Box::new(filter_by.or(series_description.gt(filter.value))),
                            Operator::Lt => Box::new(filter_by.or(series_description.lt(filter.value))),
                            Operator::Gte => Box::new(filter_by.or(series_description.ge(filter.value))),
                            Operator::Lte => Box::new(filter_by.or(series_description.le(filter.value))),
                            Operator::Ilike => {
                                Box::new(filter_by.or(series_description.ilike(format!("%{}%", filter.value))))
                            }
                        },
                        SeriesField::SeriesCfpUrl => match filter.operator {
                            Operator::Eq => Box::new(filter_by.or(series_cfp_url.eq(filter.value))),
                            Operator::Neq => Box::new(filter_by.or(series_cfp_url.ne(filter.value))),
                            Operator::Gt => Box::new(filter_by.or(series_cfp_url.gt(filter.value))),
                            Operator::Lt => Box::new(filter_by.or(series_cfp_url.lt(filter.value))),
                            Operator::Gte => Box::new(filter_by.or(series_cfp_url.ge(filter.value))),
                            Operator::Lte => Box::new(filter_by.or(series_cfp_url.le(filter.value))),
                            Operator::Ilike => {
                                Box::new(filter_by.or(series_cfp_url.ilike(format!("%{}%", filter.value))))
                            }
                        },
                        SeriesField::CreatedAt => filter_by,
                        SeriesField::UpdatedAt => filter_by,
                    };
                } else {
                    if prev_value.is_some() {
                        query = query.filter(filter_by);
                        filter_by = Box::new(series_id.eq(series_id).nullable());
                    }
                    filter_by = match filter.field {
                        // Filtering only supported for text fields
                        SeriesField::SeriesId => filter_by,
                        SeriesField::SeriesType => filter_by,
                        SeriesField::SeriesName => match filter.operator {
                            Operator::Eq => Box::new(series_name.eq(filter.value).nullable()),
                            Operator::Neq => Box::new(series_name.ne(filter.value).nullable()),
                            Operator::Gt => Box::new(series_name.gt(filter.value).nullable()),
                            Operator::Lt => Box::new(series_name.lt(filter.value).nullable()),
                            Operator::Gte => Box::new(series_name.ge(filter.value).nullable()),
                            Operator::Lte => Box::new(series_name.le(filter.value).nullable()),
                            Operator::Ilike => {
                                Box::new(series_name.ilike(format!("%{}%", filter.value)).nullable())
                            }
                        },
                        SeriesField::IssnPrint => match filter.operator {
                            Operator::Eq => Box::new(issn_print.eq(filter.value).nullable()),
                            Operator::Neq => Box::new(issn_print.ne(filter.value).nullable()),
                            Operator::Gt => Box::new(issn_print.gt(filter.value).nullable()),
                            Operator::Lt => Box::new(issn_print.lt(filter.value).nullable()),
                            Operator::Gte => Box::new(issn_print.ge(filter.value).nullable()),
                            Operator::Lte => Box::new(issn_print.le(filter.value).nullable()),
                            Operator::Ilike => {
                                Box::new(issn_print.ilike(format!("%{}%", filter.value)).nullable())
                            }
                        },
                        SeriesField::IssnDigital => match filter.operator {
                            Operator::Eq => Box::new(issn_digital.eq(filter.value).nullable()),
                            Operator::Neq => Box::new(issn_digital.ne(filter.value).nullable()),
                            Operator::Gt => Box::new(issn_digital.gt(filter.value).nullable()),
                            Operator::Lt => Box::new(issn_digital.lt(filter.value).nullable()),
                            Operator::Gte => Box::new(issn_digital.ge(filter.value).nullable()),
                            Operator::Lte => Box::new(issn_digital.le(filter.value).nullable()),
                            Operator::Ilike => {
                                Box::new(issn_digital.ilike(format!("%{}%", filter.value)).nullable())
                            }
                        },
                        SeriesField::SeriesUrl => match filter.operator {
                            Operator::Eq => Box::new(series_url.eq(filter.value)),
                            Operator::Neq => Box::new(series_url.ne(filter.value)),
                            Operator::Gt => Box::new(series_url.gt(filter.value)),
                            Operator::Lt => Box::new(series_url.lt(filter.value)),
                            Operator::Gte => Box::new(series_url.ge(filter.value)),
                            Operator::Lte => Box::new(series_url.le(filter.value)),
                            Operator::Ilike => {
                                Box::new(series_url.ilike(format!("%{}%", filter.value)))
                            }
                        },
                        SeriesField::SeriesDescription => match filter.operator {
                            Operator::Eq => Box::new(series_description.eq(filter.value)),
                            Operator::Neq => Box::new(series_description.ne(filter.value)),
                            Operator::Gt => Box::new(series_description.gt(filter.value)),
                            Operator::Lt => Box::new(series_description.lt(filter.value)),
                            Operator::Gte => Box::new(series_description.ge(filter.value)),
                            Operator::Lte => Box::new(series_description.le(filter.value)),
                            Operator::Ilike => {
                                Box::new(series_description.ilike(format!("%{}%", filter.value)))
                            }
                        },
                        SeriesField::SeriesCfpUrl => match filter.operator {
                            Operator::Eq => Box::new(series_cfp_url.eq(filter.value)),
                            Operator::Neq => Box::new(series_cfp_url.ne(filter.value)),
                            Operator::Gt => Box::new(series_cfp_url.gt(filter.value)),
                            Operator::Lt => Box::new(series_cfp_url.lt(filter.value)),
                            Operator::Gte => Box::new(series_cfp_url.ge(filter.value)),
                            Operator::Lte => Box::new(series_cfp_url.le(filter.value)),
                            Operator::Ilike => {
                                Box::new(series_cfp_url.ilike(format!("%{}%", filter.value)))
                            }
                        },
                        SeriesField::CreatedAt => filter_by,
                        SeriesField::UpdatedAt => filter_by,
                    };
                }
                prev_value = Some(curr_value.clone());
            }
        }
        match query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Series>(&mut connection)
        {
            Ok(t) => Ok(t),
            Err(e) => Err(ThothError::from(e)),
        }
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
        let mut connection = db.get().unwrap();
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
        match query.count().get_result::<i64>(&mut connection) {
            Ok(t) => Ok(t.to_string().parse::<i32>().unwrap()),
            Err(e) => Err(ThothError::from(e)),
        }
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::imprint::Imprint::from_id(db, &self.imprint_id)?.publisher_id(db)
    }

    crud_methods!(series::table, series::dsl::series);
}

impl HistoryEntry for Series {
    type NewHistoryEntity = NewSeriesHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            series_id: self.series_id,
            account_id: *account_id,
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
        let account_id: Uuid = Default::default();
        let new_series_history = series.new_history_entry(&account_id);
        assert_eq!(new_series_history.series_id, series.series_id);
        assert_eq!(new_series_history.account_id, account_id);
        assert_eq!(
            new_series_history.data,
            serde_json::Value::String(serde_json::to_string(&series).unwrap())
        );
    }
}
