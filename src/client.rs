use std::fmt;

use chrono::naive::NaiveDate;
use graphql_client::{GraphQLQuery, Response};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/work_query.graphql",
    response_derives = "Debug"
)]
pub struct WorkQuery;

#[tokio::main]
async fn query_work(
    work_id: Uuid,
    thoth_url: String,
) -> Result<Response<work_query::ResponseData>, failure::Error> {
    let request_body = WorkQuery::build_query(work_query::Variables { work_id });
    let client = reqwest::Client::new();
    let res = client.post(&thoth_url).json(&request_body).send().await?;
    let response_body: Response<work_query::ResponseData> = res.json().await?;
    Ok(response_body)
}

pub fn get_work(work_id: Uuid, thoth_url: String) -> work_query::WorkQueryWork {
    let response = query_work(work_id, thoth_url).unwrap();
    if let Some(errors) = response.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }
    let response_data: work_query::ResponseData = response.data.expect("missing response data");
    response_data.work
}

impl fmt::Display for work_query::LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
