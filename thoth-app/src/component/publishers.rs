use crate::models::publisher::publishers_query::FetchPublishers;
use crate::models::publisher::publishers_query::PublishersRequestBody;
use crate::models::publisher::publishers_query::Variables;
use crate::models::publisher::publishers_query::PublishersRequest;
use crate::models::publisher::publishers_query::FetchActionPublishers;
use crate::models::publisher::Publisher;

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
    NewPublisher,
    SEARCH_PUBLISHERS,
    PAGINATION_COUNT_PUBLISHERS,
    vec!["ID".to_string(), "Name".to_string(), "ShortName".to_string(), "URL".to_string()]
}
