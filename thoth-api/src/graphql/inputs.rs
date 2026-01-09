use crate::model::contribution::ContributionField;
use crate::model::funding::FundingField;
use crate::model::issue::IssueField;
use crate::model::language::LanguageField;
use crate::model::price::PriceField;
use crate::model::subject::SubjectField;
use crate::model::Timestamp;
use serde::Deserialize;
use serde::Serialize;

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
