use crate::onix::generate_onix_3;
use crate::SpecificationId;
use thoth_api::errors::ThothResult;
use thoth_client::work::work_query::WorkQueryWork;
use thoth_client::work::works_query::WorksQueryWorks;

pub trait AsRecord {}
impl AsRecord for WorkQueryWork {}
impl AsRecord for WorksQueryWorks {}

pub(crate) struct MetadataRecord<T: AsRecord> {
    data: T,
    specification: SpecificationId,
}

impl<T> MetadataRecord<T>
where
    T: AsRecord,
{
    pub(crate) fn new(specification: SpecificationId, data: T) -> Self {
        MetadataRecord {
            data,
            specification,
        }
    }
}

impl MetadataRecord<WorkQueryWork> {
    pub fn generate(self) -> ThothResult<String> {
        match self.specification {
            SpecificationId::Onix3ProjectMuse => generate_onix_3(self.data),
            SpecificationId::CsvThoth => unimplemented!(),
        }
    }
}

impl MetadataRecord<WorksQueryWorks> {}
