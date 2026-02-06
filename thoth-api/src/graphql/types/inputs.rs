use crate::model::contribution::ContributionField;
use crate::model::funding::FundingField;
use crate::model::issue::IssueField;
use crate::model::language::LanguageField;
use crate::model::price::PriceField;
use crate::model::subject::SubjectField;
use crate::model::Timestamp;
use serde::Deserialize;
use serde::Serialize;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(description = "Order in which to sort query results")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    #[cfg_attr(feature = "backend", graphql(description = "Ascending order"))]
    #[default]
    Asc,
    #[cfg_attr(feature = "backend", graphql(description = "Descending order"))]
    Desc,
}

#[test]
fn test_direction_default() {
    let dir: Direction = Default::default();
    assert_eq!(dir, Direction::Asc);
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(description = "Expression to use when filtering by numeric value")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Expression {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Return only results with values which are greater than the value supplied"
        )
    )]
    #[default]
    GreaterThan,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Return only results with values which are less than the value supplied"
        )
    )]
    LessThan,
}

#[test]
fn test_expression_default() {
    let dir: Expression = Default::default();
    assert_eq!(dir, Expression::GreaterThan);
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting contributions list")]
pub struct ContributionOrderBy {
    pub field: ContributionField,
    pub direction: Direction,
}

impl Default for ContributionOrderBy {
    fn default() -> ContributionOrderBy {
        ContributionOrderBy {
            field: ContributionField::ContributionType,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting issues list")]
pub struct IssueOrderBy {
    pub field: IssueField,
    pub direction: Direction,
}

impl Default for IssueOrderBy {
    fn default() -> IssueOrderBy {
        IssueOrderBy {
            field: IssueField::IssueOrdinal,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting languages list")]
pub struct LanguageOrderBy {
    pub field: LanguageField,
    pub direction: Direction,
}

impl Default for LanguageOrderBy {
    fn default() -> LanguageOrderBy {
        LanguageOrderBy {
            field: LanguageField::LanguageCode,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting prices list")]
pub struct PriceOrderBy {
    pub field: PriceField,
    pub direction: Direction,
}

impl Default for PriceOrderBy {
    fn default() -> PriceOrderBy {
        PriceOrderBy {
            field: PriceField::CurrencyCode,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting subjects list")]
pub struct SubjectOrderBy {
    pub field: SubjectField,
    pub direction: Direction,
}

impl Default for SubjectOrderBy {
    fn default() -> SubjectOrderBy {
        SubjectOrderBy {
            field: SubjectField::SubjectType,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Field and order to use when sorting fundings list")]
pub struct FundingOrderBy {
    pub field: FundingField,
    pub direction: Direction,
}

impl Default for FundingOrderBy {
    fn default() -> FundingOrderBy {
        FundingOrderBy {
            field: FundingField::Program,
            direction: Default::default(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(
    description = "Timestamp and choice out of greater than/less than to use when filtering by a time field (e.g. updated_at)"
)]
pub struct TimeExpression {
    pub timestamp: Timestamp,
    pub expression: Expression,
}

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    EnumString,
    Display,
    juniper::GraphQLEnum,
)]
#[graphql(description = "Unit of measurement for physical Work dimensions (mm, cm or in)")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "lowercase")]
pub enum LengthUnit {
    #[cfg_attr(feature = "backend", graphql(description = "Millimetres"))]
    #[default]
    Mm,
    #[cfg_attr(feature = "backend", graphql(description = "Centimetres"))]
    Cm,
    #[cfg_attr(feature = "backend", graphql(description = "Inches"))]
    In,
}

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    EnumString,
    Display,
    juniper::GraphQLEnum,
)]
#[graphql(description = "Unit of measurement for physical Work weight (grams or ounces)")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "lowercase")]
pub enum WeightUnit {
    #[cfg_attr(feature = "backend", graphql(description = "Grams"))]
    #[default]
    G,
    #[cfg_attr(feature = "backend", graphql(description = "Ounces"))]
    Oz,
}

pub trait Convert {
    fn convert_length_from_to(&self, current_units: &LengthUnit, new_units: &LengthUnit) -> f64;
    fn convert_weight_from_to(&self, current_units: &WeightUnit, new_units: &WeightUnit) -> f64;
}

impl Convert for f64 {
    fn convert_length_from_to(&self, current_units: &LengthUnit, new_units: &LengthUnit) -> f64 {
        match (current_units, new_units) {
            // If current units and new units are the same, no conversion is needed
            (LengthUnit::Mm, LengthUnit::Mm)
            | (LengthUnit::Cm, LengthUnit::Cm)
            | (LengthUnit::In, LengthUnit::In) => *self,
            // Return cm values rounded to max 1 decimal place (1 cm = 10 mm)
            (LengthUnit::Mm, LengthUnit::Cm) => self.round() / 10.0,
            // Return mm values rounded to nearest mm (1 cm = 10 mm)
            (LengthUnit::Cm, LengthUnit::Mm) => (self * 10.0).round(),
            // Return inch values rounded to 2 decimal places (1 inch = 25.4 mm)
            (LengthUnit::Mm, LengthUnit::In) => {
                let unrounded_inches = self / 25.4;
                // To round to a non-integer scale, multiply by the appropriate factor,
                // round to the nearest integer, then divide again by the same factor
                (unrounded_inches * 100.0).round() / 100.0
            }
            // Return mm values rounded to nearest mm (1 inch = 25.4 mm)
            (LengthUnit::In, LengthUnit::Mm) => (self * 25.4).round(),
            // We don't currently support conversion between cm and in as it is not required
            _ => unimplemented!(),
        }
    }

    fn convert_weight_from_to(&self, current_units: &WeightUnit, new_units: &WeightUnit) -> f64 {
        match (current_units, new_units) {
            // If current units and new units are the same, no conversion is needed
            (WeightUnit::G, WeightUnit::G) | (WeightUnit::Oz, WeightUnit::Oz) => *self,
            // Return ounce values rounded to 4 decimal places (1 ounce = 28.349523125 grams)
            (WeightUnit::G, WeightUnit::Oz) => {
                let unrounded_ounces = self / 28.349523125;
                // To round to a non-integer scale, multiply by the appropriate factor,
                // round to the nearest integer, then divide again by the same factor
                (unrounded_ounces * 10000.0).round() / 10000.0
            }
            // Return gram values rounded to nearest gram (1 ounce = 28.349523125 grams)
            (WeightUnit::Oz, WeightUnit::G) => (self * 28.349523125).round(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Convert, LengthUnit::*, WeightUnit::*};

    #[test]
    // Float equality comparison is fine here because the floats
    // have already been rounded by the functions under test
    #[allow(clippy::float_cmp)]
    fn test_convert_length_from_to() {
        assert_eq!(123.456.convert_length_from_to(&Mm, &Cm), 12.3);
        assert_eq!(123.456.convert_length_from_to(&Mm, &In), 4.86);
        assert_eq!(123.456.convert_length_from_to(&Cm, &Mm), 1235.0);
        assert_eq!(123.456.convert_length_from_to(&In, &Mm), 3136.0);
        // Test some standard print sizes
        assert_eq!(4.25.convert_length_from_to(&In, &Mm), 108.0);
        assert_eq!(108.0.convert_length_from_to(&Mm, &In), 4.25);
        assert_eq!(6.0.convert_length_from_to(&In, &Mm), 152.0);
        assert_eq!(152.0.convert_length_from_to(&Mm, &In), 5.98);
        assert_eq!(8.5.convert_length_from_to(&In, &Mm), 216.0);
        assert_eq!(216.0.convert_length_from_to(&Mm, &In), 8.5);
        // Test that converting and then converting back again
        // returns a value within a reasonable margin of error
        assert_eq!(
            5.06.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            5.08
        );
        assert_eq!(
            6.5.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            6.5
        );
        assert_eq!(
            7.44.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            7.44
        );
        assert_eq!(
            8.27.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            8.27
        );
        assert_eq!(
            9.0.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            9.02
        );
        assert_eq!(
            10.88
                .convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            10.87
        );
        assert_eq!(
            102.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            102.0
        );
        assert_eq!(
            120.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            120.0
        );
        assert_eq!(
            168.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            168.0
        );
        assert_eq!(
            190.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            190.0
        );
    }
    #[test]
    // Float equality comparison is fine here because the floats
    // have already been rounded by the functions under test
    #[allow(clippy::float_cmp)]
    fn test_convert_weight_from_to() {
        assert_eq!(123.456.convert_weight_from_to(&G, &Oz), 4.3548);
        assert_eq!(123.456.convert_weight_from_to(&Oz, &G), 3500.0);
        assert_eq!(4.25.convert_weight_from_to(&Oz, &G), 120.0);
        assert_eq!(108.0.convert_weight_from_to(&G, &Oz), 3.8096);
        assert_eq!(6.0.convert_weight_from_to(&Oz, &G), 170.0);
        assert_eq!(152.0.convert_weight_from_to(&G, &Oz), 5.3616);
        assert_eq!(8.5.convert_weight_from_to(&Oz, &G), 241.0);
        assert_eq!(216.0.convert_weight_from_to(&G, &Oz), 7.6192);
        // Test that converting and then converting back again
        // returns a value within a reasonable margin of error
        assert_eq!(
            5.0.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            5.0089
        );
        assert_eq!(
            5.125
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            5.1147
        );
        assert_eq!(
            6.5.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            6.4904
        );
        assert_eq!(
            7.25.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            7.2664
        );
        assert_eq!(
            7.44.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            7.4428
        );
        assert_eq!(
            8.0625
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            8.0777
        );
        assert_eq!(
            9.0.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            8.9949
        );
        assert_eq!(
            10.75
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            10.7586
        );
        assert_eq!(
            10.88
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            10.8644
        );
        assert_eq!(
            102.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            102.0
        );
        assert_eq!(
            120.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            120.0
        );
        assert_eq!(
            168.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            168.0
        );
        assert_eq!(
            190.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            190.0
        );
    }
}
