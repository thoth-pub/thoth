use crate::models::publication::publications_query::FetchActionPublications;
use crate::models::publication::publications_query::FetchPublications;
use crate::models::publication::publications_query::PublicationsRequest;
use crate::models::publication::publications_query::PublicationsRequestBody;
use crate::models::publication::publications_query::Variables;
use thoth_api::publication::model::PublicationField;
use thoth_api::publication::model::PublicationOrderBy;
use thoth_api::publication::model::PublicationWithRelations;

pagination_component! {
    PublicationsComponent,
    PublicationWithRelations,
    publications,
    publication_count,
    PublicationsRequest,
    FetchActionPublications,
    FetchPublications,
    PublicationsRequestBody,
    Variables,
    SEARCH_PUBLICATIONS,
    PAGINATION_COUNT_PUBLICATIONS,
    vec![
        PublicationField::PublicationId.to_string(),
        "Work Title".to_string(),
        "Work DOI".to_string(),
        "Publisher".to_string(),
        PublicationField::PublicationType.to_string(),
        PublicationField::Isbn.to_string(),
        PublicationField::PublicationUrl.to_string(),
        PublicationField::UpdatedAt.to_string(),
    ],
    PublicationOrderBy,
    PublicationField,
}
