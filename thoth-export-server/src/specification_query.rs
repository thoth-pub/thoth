use futures::{stream, StreamExt, TryStreamExt};
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use thoth_client::{QueryParameters, ThothClient, Work};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::record::MetadataSpecification;

const CONCURRENT_REQUESTS: usize = 4;
const PAGINATION_LIMIT: i64 = 100;

#[derive(Copy, Clone)]
enum SpecificationRequest {
    ByWork,
    ByPublisher,
}

#[derive(Clone)]
pub(crate) struct SpecificationQuery {
    id: Uuid,
    thoth_client: Arc<ThothClient>,
    query_configuration: QueryConfiguration,
}

#[derive(Copy, Clone)]
struct QueryConfiguration {
    request: SpecificationRequest,
    specification: MetadataSpecification,
}

impl SpecificationQuery {
    pub(crate) fn by_work(
        thoth_client: Arc<ThothClient>,
        id: Uuid,
        specification: MetadataSpecification,
    ) -> Self {
        let query_configuration = QueryConfiguration::by_work(specification);
        Self {
            id,
            thoth_client,
            query_configuration,
        }
    }

    pub(crate) fn by_publisher(
        thoth_client: Arc<ThothClient>,
        id: Uuid,
        specification: MetadataSpecification,
    ) -> Self {
        let query_configuration = QueryConfiguration::by_publisher(specification);
        Self {
            id,
            thoth_client,
            query_configuration,
        }
    }

    pub(crate) async fn run(self) -> ThothResult<Vec<Work>> {
        let parameters: QueryParameters = self.query_configuration.try_into()?;
        match self.query_configuration.request {
            SpecificationRequest::ByWork => self
                .thoth_client
                .get_work(self.id, parameters)
                .await
                .map(|w| vec![w]),
            SpecificationRequest::ByPublisher => {
                // get the total work count to figure out how to paginate the results
                let work_count = self
                    .thoth_client
                    .get_work_count(Some(vec![self.id]))
                    .await?;
                // calculate total pages, rounding up to ensure all works are covered
                let total_pages = (work_count + PAGINATION_LIMIT - 1) / PAGINATION_LIMIT;
                // get a vector of all page offsets we will need
                let offsets = (1..=total_pages) // inclusive upper bound
                    .map(|current_page| (current_page - 1) * PAGINATION_LIMIT)
                    .collect::<Vec<i64>>();

                // make concurrent requests iterating the list of offsets to asynchronously obtain all pages
                let mut works_pages = stream::iter(offsets)
                    .map(|offset| {
                        let client = &self.thoth_client;
                        async move {
                            client
                                .get_works(
                                    Some(vec![self.id]),
                                    PAGINATION_LIMIT,
                                    offset,
                                    parameters,
                                )
                                .await
                        }
                    })
                    .buffer_unordered(CONCURRENT_REQUESTS);

                // merge all pages
                let mut works: Vec<Work> = vec![];
                while let Some(page) = works_pages.try_next().await? {
                    works.extend(page);
                }
                Ok(works)
            }
        }
    }
}

impl QueryConfiguration {
    fn by_work(specification: MetadataSpecification) -> Self {
        Self {
            request: SpecificationRequest::ByWork,
            specification,
        }
    }

    fn by_publisher(specification: MetadataSpecification) -> Self {
        Self {
            request: SpecificationRequest::ByPublisher,
            specification,
        }
    }
}

impl TryFrom<QueryConfiguration> for QueryParameters {
    type Error = ThothError;

    fn try_from(q: QueryConfiguration) -> ThothResult<Self> {
        match q.specification {
            MetadataSpecification::Onix31Thoth(_) => Ok(QueryParameters::new().with_all()),
            MetadataSpecification::Onix3Thoth(_) => Ok(QueryParameters::new().with_all()),
            MetadataSpecification::Onix3ProjectMuse(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::Onix3Oapen(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::Onix3Jstor(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::Onix3GoogleBooks(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::Onix3Overdrive(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::Onix21EbscoHost(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::Onix21ProquestEbrary(_) => Ok(QueryParameters::new()
                .with_all()
                .without_relations()
                .without_references()),
            MetadataSpecification::CsvThoth(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new()
                    .with_all()
                    .without_relations()
                    .without_references()),
            },
            MetadataSpecification::JsonThoth(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Err(ThothError::IncompleteMetadataRecord(
                    "json::thoth".to_string(),
                    "Output can only be generated for one work at a time".to_string(),
                )),
            },
            MetadataSpecification::KbartOclc(_) => {
                Ok(QueryParameters::new().with_issues().with_publications())
            }
            MetadataSpecification::BibtexThoth(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new()
                    .with_issues()
                    .with_publications()
                    .with_relations()),
                SpecificationRequest::ByPublisher => {
                    Ok(QueryParameters::new().with_issues().with_publications())
                }
            },
            MetadataSpecification::DoiDepositCrossref(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new()
                    .with_issues()
                    .with_publications()
                    .with_fundings()
                    .with_relations()
                    .with_references()),
                SpecificationRequest::ByPublisher => Err(ThothError::IncompleteMetadataRecord(
                    "doideposit::crossref".to_string(),
                    "Output can only be generated for one work at a time".to_string(),
                )),
            },
            MetadataSpecification::Marc21RecordThoth(_)
            | MetadataSpecification::Marc21MarkupThoth(_)
            | MetadataSpecification::Marc21XmlThoth(_) => Ok(QueryParameters::new()
                .with_issues()
                .with_publications()
                .with_subjects()
                .with_languages()
                .with_fundings()),
        }
    }
}
