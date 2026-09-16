//! API-owned work-level incremental distribution (`BE-06`, #848): the generic
//! `WORK_UPSERT` substrate over the released `BE-04` durable jobs.
//!
//! The specification is `docs/publisher-services/specifications/BE-06-R52B.md`
//! as amended by #848 Amendments 1-3. Crossref is the only implemented
//! work-level execution profile; its permit, timestamp and version-floor model
//! lives in [`crate::model::crossref_write_permit`].
//!
//! Everything here is Publisher-Services-specific. There is no generic
//! cross-programme queue, scheduler or job framework (frozen rule 1, R52B §5).

#[cfg(all(test, feature = "backend"))]
mod tests;
