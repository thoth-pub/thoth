//! The Crossref execution profile of `BE-06` (#848): universal write permits,
//! canonical DOI membership, deposit timestamps and the version floor.
//!
//! The specification is `docs/publisher-services/specifications/BE-06-R52B.md`
//! sections 14-17 as amended by #848 Amendments 1-3. Nothing here contacts
//! Crossref: provider writes are performed by the separately specified
//! downstream dissemination task (`thoth-pub/thoth-dissemination#106`).

#[cfg(feature = "backend")]
pub mod crud;

#[cfg(all(test, feature = "backend"))]
mod tests;
