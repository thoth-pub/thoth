use serde::Deserialize;
use serde::Serialize;

pub const ONIX_MAX_CHAR_LIMIT: u16 = 350;

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
