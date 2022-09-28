use std::convert::TryFrom;
use thoth_client::QueryParameters;
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
    pub(crate) fn by_work(specification: MetadataSpecification) -> Self {
        Self {
            request: SpecificationRequest::ByWork,
            specification,
        }
    }

    pub(crate) fn by_publisher(specification: MetadataSpecification) -> Self {
        Self {
            request: SpecificationRequest::ByPublisher,
            specification,
        }
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
