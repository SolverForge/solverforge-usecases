use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed spare-part inventory fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Part {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub initial_on_hand: i32,
}

impl Part {
    pub fn new(id: impl Into<String>, name: impl Into<String>, initial_on_hand: i32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            initial_on_hand,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_construction() {
        let fact = Part::new("test-id", "test", 1);
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.initial_on_hand;
    }
}
