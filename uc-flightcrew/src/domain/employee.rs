use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Crew member with a home base, qualification, and calendar unavailability.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Employee {
    #[planning_id]
    pub id: String,
    pub name: String,
    pub home_airport_idx: usize,
    pub skills: Vec<String>,
    /// Whole planning-horizon day indexes on which this employee cannot work.
    pub unavailable_days: Vec<i64>,
    #[serde(skip)]
    pub index: usize,
}

impl Employee {
    pub fn new(id: impl Into<String>, name: impl Into<String>, home_airport_idx: usize, skills: Vec<String>, unavailable_days: Vec<i64>) -> Self {
        Self { id: id.into(), name: name.into(), home_airport_idx, skills, unavailable_days, index: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_construction() {
        let fact = Employee::new("test-id", "test", Default::default(), Default::default(), Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.home_airport_idx;
        let _ = &fact.skills;
        let _ = &fact.unavailable_days;
    }
}
