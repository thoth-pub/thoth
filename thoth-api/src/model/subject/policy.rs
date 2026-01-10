use crate::model::subject::{thema::THEMA_CODES, NewSubject, PatchSubject, Subject, SubjectType};
use crate::policy::{CreatePolicy, DeletePolicy, MovePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Subject`.
///
/// For now this policy enforces the tenant boundary only:
/// - authentication
/// - publisher membership derived from the entity / input via `PublisherId`
pub struct SubjectPolicy;

fn check_subject(subject_type: &SubjectType, code: &str) -> ThothResult<()> {
    if matches!(subject_type, SubjectType::Thema) && THEMA_CODES.binary_search(&code).is_err() {
        return Err(ThothError::InvalidSubjectCode {
            input: code.to_string(),
            subject_type: subject_type.to_string(),
        });
    }
    Ok(())
}

impl CreatePolicy<NewSubject> for SubjectPolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewSubject, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        check_subject(&data.subject_type, &data.subject_code)
    }
}

impl UpdatePolicy<Subject, PatchSubject> for SubjectPolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Subject,
        patch: &PatchSubject,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;
        check_subject(&patch.subject_type, &patch.subject_code)
    }
}

impl DeletePolicy<Subject> for SubjectPolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Subject) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}

impl MovePolicy<Subject> for SubjectPolicy {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Subject) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}

#[test]
fn test_check_subject() {
    // Valid codes for specific schemas
    assert!(check_subject(&SubjectType::Bic, "HRQX9").is_ok());
    assert!(check_subject(&SubjectType::Bisac, "BIB004060").is_ok());
    assert!(check_subject(&SubjectType::Thema, "ATXZ1").is_ok());

    // Custom fields: no validity restrictions
    assert!(check_subject(&SubjectType::Custom, "A custom subject").is_ok());
    assert!(check_subject(&SubjectType::Keyword, "keyword").is_ok());

    // Invalid codes for specific schemas: only validate Thema
    assert!(check_subject(&SubjectType::Bic, "ABCD0").is_ok());
    assert!(check_subject(&SubjectType::Bisac, "BLA123456").is_ok());
    assert!(check_subject(&SubjectType::Thema, "AHBW").is_err());
}
