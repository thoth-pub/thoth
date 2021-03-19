use crate::models::publisher::publishers_query::FetchActionPublishers;
use crate::models::publisher::publishers_query::FetchPublishers;
use crate::models::publisher::publishers_query::PublishersRequest;
use crate::models::publisher::publishers_query::PublishersRequestBody;
use crate::models::publisher::publishers_query::Variables;
use crate::models::publisher::Publisher;
use thoth_api::publisher::model::PublisherField;
use thoth_api::publisher::model::PublisherOrderBy;

pagination_component! {
    PublishersComponent,
    Publisher,
    publishers,
    publisher_count,
    PublishersRequest,
    FetchActionPublishers,
    FetchPublishers,
    PublishersRequestBody,
    Variables,
    SEARCH_PUBLISHERS,
    PAGINATION_COUNT_PUBLISHERS,
    vec!["ID".to_string(), "Name".to_string(), "ShortName".to_string(), "URL".to_string(), "Updated".to_string()],
    PublisherOrderBy,
    PublisherField,
}
