use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed planning-horizon day fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Day {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub index: i32,
    pub week: i32,
}

impl Day {
    pub fn new(id: impl Into<String>, name: impl Into<String>, index: i32, week: i32) -> Self {
        Self { id: id.into(), name: name.into(), index, week }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_construction() {
        let fact = Day::new("test-id", "test", Default::default(), Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.index;
        let _ = &fact.week;
    }
}
