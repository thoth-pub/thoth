use crate::model::price::{NewPrice, PatchPrice, Price};
use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, UpdatePolicy};
use thoth_errors::{ThothError, ThothResult};

/// Write policies for `Price`.
///
/// These policies are responsible for:
/// - requiring authentication
/// - requiring publisher membership (tenant boundary)
/// - enforcing business rules (e.g. non-zero unit price)
pub struct PricePolicy;

fn validate_unit_price(unit_price: f64) -> ThothResult<()> {
    // Prices must be non-zero (and non-negative).
    if unit_price <= 0.0 {
        return Err(ThothError::PriceZeroError);
    }
    Ok(())
}

impl CreatePolicy<NewPrice> for PricePolicy {
    fn can_create<C: PolicyContext>(ctx: &C, data: &NewPrice, _params: ()) -> ThothResult<()> {
        ctx.require_publisher_for(data)?;
        validate_unit_price(data.unit_price)
    }
}

impl UpdatePolicy<Price, PatchPrice> for PricePolicy {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Price,
        patch: &PatchPrice,
        _params: (),
    ) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        ctx.require_publisher_for(patch)?;

        // Enforce non-zero unit price.
        validate_unit_price(patch.unit_price)
    }
}

impl DeletePolicy<Price> for PricePolicy {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Price) -> ThothResult<()> {
        ctx.require_publisher_for(current)?;
        Ok(())
    }
}
