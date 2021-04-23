use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::schema::series_history;
use crate::series::model::NewSeriesHistory;
use crate::series::model::Series;
use crate::series::model::SeriesHistory;

impl NewSeriesHistory {
    pub fn new(series: Series, account_id: Uuid) -> Self {
        Self {
            series_id: series.series_id,
            account_id,
            data: serde_json::Value::String(serde_json::to_string(&series).unwrap()),
        }
    }

    pub fn insert(&self, connection: &PgConnection) -> ThothResult<SeriesHistory> {
        match diesel::insert_into(series_history::table)
            .values(self)
            .get_result(connection)
        {
            Ok(history) => Ok(history),
            Err(e) => Err(ThothError::from(e)),
        }
    }
}
