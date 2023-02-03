use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use thoth_client::{QueryParameters, ThothClient, Work};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::record::MetadataSpecification;

enum SpecificationRequest {
    ByWork,
    ByPublisher,
}

pub(crate) struct SpecificationQuery {
    thoth_client: Arc<ThothClient>,
    specification: MetadataSpecification,
}

struct QueryConfiguration {
    request: SpecificationRequest,
    specification: MetadataSpecification,
}

impl SpecificationQuery {
    pub(crate) fn new(
        thoth_client: Arc<ThothClient>,
        specification: MetadataSpecification,
    ) -> Self {
        Self {
            thoth_client,
            specification,
        }
    }

    pub(crate) async fn by_work(self, work_id: Uuid) -> ThothResult<Work> {
        let parameters: QueryParameters =
            QueryConfiguration::by_work(self.specification).try_into()?;
        self.thoth_client.get_work(work_id, parameters).await
    }

    pub(crate) async fn by_publisher(self, publisher_id: Uuid) -> ThothResult<Vec<Work>> {
        let parameters: QueryParameters =
            QueryConfiguration::by_publisher(self.specification).try_into()?;
        self.thoth_client
            .get_works(Some(vec![publisher_id]), parameters)
            .await
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
            MetadataSpecification::Marc21RecordThoth(_) => {
                Ok(QueryParameters::new().with_issues().with_publications())
            }
        }
    }
}
