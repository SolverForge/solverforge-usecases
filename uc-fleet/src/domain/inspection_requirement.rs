use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed inspection requirement fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct InspectionRequirement {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub vessel_id: String,
    pub duration_days: i32,
    pub earliest_day: i32,
    pub latest_day: i32,
    pub required_qa_per_day: i32,
}

impl InspectionRequirement {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        vessel_id: impl Into<String>,
        duration_days: i32,
        earliest_day: i32,
        latest_day: i32,
        required_qa_per_day: i32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            vessel_id: vessel_id.into(),
            duration_days,
            earliest_day,
            latest_day,
            required_qa_per_day,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspection_requirement_construction() {
        let fact = InspectionRequirement::new("test-id", "test", "PC-01", 1, 5, 10, 1);
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.vessel_id;
        let _ = &fact.duration_days;
        let _ = &fact.earliest_day;
        let _ = &fact.latest_day;
        let _ = &fact.required_qa_per_day;
    }
}
