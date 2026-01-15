use crate::model::language::{Language, NewLanguage, PatchLanguage};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::ThothResult;

/// Write policies for `Language`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
pub struct LanguagePolicy;

impl CreatePolicy<NewLanguage> for LanguagePolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewLanguage, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        Ok(())
    }
}

impl UpdatePolicy<Language, PatchLanguage> for LanguagePolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Language,
        patch: &PatchLanguage,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        Ok(())
    }
}

impl DeletePolicy<Language> for LanguagePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Language) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
