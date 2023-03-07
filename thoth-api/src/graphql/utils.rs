use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(description = "Order in which to sort query results (ascending or descending)")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Asc,
    Desc,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Asc
    }
}

#[test]
fn test_direction_default() {
    let dir: Direction = Default::default();
    assert_eq!(dir, Direction::Asc);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(
    description = "Expression to use when filtering by numeric value (greater than or less than)"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Expression {
    GreaterThan,
    LessThan,
}

impl Default for Expression {
    fn default() -> Expression {
        Expression::GreaterThan
    }
}

#[test]
fn test_expression_default() {
    let dir: Expression = Default::default();
    assert_eq!(dir, Expression::GreaterThan);
}
