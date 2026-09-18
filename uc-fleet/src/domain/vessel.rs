use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed vessel facts for the synthetic readiness planning scenario.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Vessel {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub vessel_class: String,
    pub compatible_dock_classes: Vec<String>,
    pub ready_from_day: i32,
}

impl Vessel {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        vessel_class: impl Into<String>,
        compatible_dock_classes: Vec<String>,
        ready_from_day: i32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            vessel_class: vessel_class.into(),
            compatible_dock_classes,
            ready_from_day,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vessel_construction() {
        let fact = Vessel::new(
            "test-id",
            "test",
            "patrol_cutter",
            vec!["small".to_string()],
            1,
        );
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.vessel_class;
        let _ = &fact.compatible_dock_classes;
        let _ = &fact.ready_from_day;
    }
}
