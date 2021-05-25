use crate::{Specification, SpecificationId};
use thoth_client::work::work_query::WorkQueryWork;
use thoth_client::work::works_query::WorksQueryWorks;
use thoth_api::errors::ThothResult;
use crate::onix::generate_onix_3;

pub trait AsRecord {}
impl AsRecord for WorkQueryWork {}
impl AsRecord for WorksQueryWorks {}

pub(crate) struct MetadataRecord<'a, T: AsRecord> {
    data: T,
    specification: &'a Specification<'a>,
}

impl<T> MetadataRecord<'_, T>
where
    T: AsRecord,
{
    pub(crate) fn new(specification: &'static Specification, data: T) -> Self {
        MetadataRecord {
            data,
            specification,
        }
    }
}


impl MetadataRecord<'_, WorkQueryWork> {
    pub fn generate(self) -> ThothResult<Vec<u8>> {
        match self.specification.id {
            SpecificationId::Onix3ProjectMuse => generate_onix_3(self.data),
            SpecificationId::CsvThoth => unimplemented!(),
        }
    }
}

impl MetadataRecord<'_, WorksQueryWorks> {}