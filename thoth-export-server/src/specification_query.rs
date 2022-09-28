use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use uuid::Uuid;
use thoth_client::{QueryParameters, ThothClient, Work};
use thoth_errors::{ThothError, ThothResult};

use crate::record::MetadataSpecification;

enum SpecificationRequest {
    ByWork,
    ByPublisher,
}

pub(crate) struct SpecificationQuery {
    request: SpecificationRequest,
    specification: MetadataSpecification,
}

impl SpecificationQuery {
    pub(crate) async fn by_work(
        thoth_client: Arc<ThothClient>,
        specification: MetadataSpecification,
        work_id: Uuid,
    ) -> ThothResult<Work> {
        let query = SpecificationQuery {
            request: SpecificationRequest::ByWork,
            specification,
        };
        thoth_client.get_work(work_id, query.try_into()?).await
    }

    pub(crate) async fn by_publisher(
        thoth_client: Arc<ThothClient>,
        specification: MetadataSpecification,
        publisher_id: Uuid,
    ) -> ThothResult<Vec<Work>> {
        let query = SpecificationQuery {
            request: SpecificationRequest::ByPublisher,
            specification,
        };
        thoth_client.get_works(Some(vec![publisher_id]), query.try_into()?).await
    }
}

impl TryFrom<SpecificationQuery> for QueryParameters {
    type Error = ThothError;

    fn try_from(q: SpecificationQuery) -> ThothResult<Self> {
        match q.specification {
            MetadataSpecification::Onix3ProjectMuse(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::Onix3Oapen(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::Onix3Jstor(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::Onix3GoogleBooks(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::Onix3Overdrive(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::Onix21EbscoHost(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::Onix21ProquestEbrary(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::CsvThoth(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::KbartOclc(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::BibtexThoth(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Ok(QueryParameters::new().with_all()),
            },
            MetadataSpecification::DoiDepositCrossref(_) => match q.request {
                SpecificationRequest::ByWork => Ok(QueryParameters::new().with_all()),
                SpecificationRequest::ByPublisher => Err(ThothError::IncompleteMetadataRecord(
                    "doideposit::crossref".to_string(),
                    "Output can only be generated for one work at a time".to_string(),
                )),
            },
        }
    }
}
