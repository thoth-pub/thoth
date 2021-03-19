use crate::models::publication::publications_query::DetailedPublication;
use crate::models::publication::publications_query::FetchActionPublications;
use crate::models::publication::publications_query::FetchPublications;
use crate::models::publication::publications_query::PublicationsRequest;
use crate::models::publication::publications_query::PublicationsRequestBody;
use crate::models::publication::publications_query::Variables;
use thoth_api::publication::model::PublicationField;
use thoth_api::publication::model::PublicationOrderBy;

pagination_component! {
    PublicationsComponent,
    DetailedPublication,
    publications,
    publication_count,
    PublicationsRequest,
    FetchActionPublications,
    FetchPublications,
    PublicationsRequestBody,
    Variables,
    SEARCH_PUBLICATIONS,
    PAGINATION_COUNT_PUBLICATIONS,
    vec!["ID".to_string(), "Work Title".to_string(), "Work DOI".to_string(), "Publisher".to_string(), "Type".to_string(), "ISBN".to_string(), "URL".to_string(), "Updated".to_string()],
    PublicationOrderBy,
    PublicationField,
}
