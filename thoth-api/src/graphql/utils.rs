use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(description = "Order in which to sort query results (ascending or descending)")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    #[default]
    Asc,
    Desc,
}

#[test]
fn test_direction_default() {
    let dir: Direction = Default::default();
    assert_eq!(dir, Direction::Asc);
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, juniper::GraphQLEnum)]
#[graphql(
    description = "Expression to use when filtering by numeric value (greater than or less than)"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Expression {
    #[default]
    GreaterThan,
    LessThan,
}

#[test]
fn test_expression_default() {
    let dir: Expression = Default::default();
    assert_eq!(dir, Expression::GreaterThan);
}
#[derive(juniper::GraphQLEnum)]
pub enum Operator {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Ilike,
}
