//! Temporary source-compatibility alias used only while the large GraphQL
//! `Context` definition is migrated in this same bounded implementation.
//!
//! The ADR-0006 store implementation, stored-result reuse, response-scope
//! identity, load-shape namespaces, failure store, and guard-conditioned
//! availability have all been removed. `GraphqlBatchStore` is now only the old
//! source-level name for the ADR-0007 request-local loader bundle. The alias and
//! old `Context.batch_store` field are removed before review.

pub(crate) type GraphqlBatchStore = super::dataloader::RequestLoaders;
