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
