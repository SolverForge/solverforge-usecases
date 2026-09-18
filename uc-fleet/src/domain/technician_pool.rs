use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Fixed technician capacity pool fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct TechnicianPool {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub skill: String,
    pub capacity_per_day: i32,
    pub overtime_capacity_per_day: i32,
}

impl TechnicianPool {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        skill: impl Into<String>,
        capacity_per_day: i32,
        overtime_capacity_per_day: i32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            skill: skill.into(),
            capacity_per_day,
            overtime_capacity_per_day,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technician_pool_construction() {
        let fact = TechnicianPool::new("test-id", "test", "PROP", 4, 1);
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.skill;
        let _ = &fact.capacity_per_day;
        let _ = &fact.overtime_capacity_per_day;
    }
}
