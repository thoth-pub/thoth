use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, juniper::GraphQLEnum)]
#[graphql(description = "Order in which to sort query results (ascending or descending)")]
pub enum Direction {
    ASC,
    DESC,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::ASC
    }
}

#[test]
fn test_direction_default() {
    let dir: Direction = Default::default();
    assert_eq!(dir, Direction::ASC);
}
