use thoth_client::QueryParameters;

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

impl From<SpecificationQuery> for QueryParameters {
    fn from(q: SpecificationQuery) -> Self {
        match q.specification {
            MetadataSpecification::Onix3ProjectMuse(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::Onix3Oapen(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::Onix3Jstor(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::Onix3GoogleBooks(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::Onix3Overdrive(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::Onix21EbscoHost(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::Onix21ProquestEbrary(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::CsvThoth(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::KbartOclc(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::BibtexThoth(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
            MetadataSpecification::DoiDepositCrossref(_) => match q.request {
                SpecificationRequest::ByWork => QueryParameters::new(),
                SpecificationRequest::ByPublisher => QueryParameters::new(),
            },
        }
    }
}
