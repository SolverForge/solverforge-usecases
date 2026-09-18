use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed dock facts and compatibility classes.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Dock {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub dock_class: String,
    pub capacity: i32,
}

impl Dock {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        dock_class: impl Into<String>,
        capacity: i32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dock_class: dock_class.into(),
            capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_construction() {
        let fact = Dock::new("test-id", "test", "large", 1);
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.dock_class;
        let _ = &fact.capacity;
    }
}
