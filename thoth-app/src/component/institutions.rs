#![allow(clippy::unnecessary_operation)]

use crate::models::institution::institutions_query::FetchActionInstitutions;
use crate::models::institution::institutions_query::FetchInstitutions;
use crate::models::institution::institutions_query::InstitutionsRequest;
use crate::models::institution::institutions_query::InstitutionsRequestBody;
use crate::models::institution::institutions_query::Variables;
use thoth_api::model::institution::Institution;
use thoth_api::model::institution::InstitutionField;
use thoth_api::model::institution::InstitutionOrderBy;

use super::ToElementValue;

pagination_component! {
    InstitutionsComponent,
    Institution,
    institutions,
    institution_count,
    InstitutionsRequest,
    FetchActionInstitutions,
    FetchInstitutions,
    InstitutionsRequestBody,
    Variables,
    SEARCH_INSTITUTIONS,
    PAGINATION_COUNT_INSTITUTIONS,
    vec![
        InstitutionField::InstitutionId.to_string(),
        InstitutionField::InstitutionName.to_string(),
        InstitutionField::InstitutionDoi.to_string(),
        InstitutionField::Ror.to_string(),
        InstitutionField::CountryCode.to_string(),
        InstitutionField::UpdatedAt.to_string(),
    ],
    InstitutionOrderBy,
    InstitutionField,
}
