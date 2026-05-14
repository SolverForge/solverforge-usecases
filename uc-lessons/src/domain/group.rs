use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// A student cohort that receives a weekly timetable.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Group {
    #[planning_id]
    pub id: String,
    #[serde(skip)]
    pub index: usize, // the solver-facing join key
    pub name: String,
    pub student_count: usize,
    pub availability: Vec<bool>,
}

impl Group {
    pub fn new(
        index: usize,
        name: impl Into<String>,
        student_count: usize,
        availability: impl Into<Vec<bool>>,
    ) -> Self {
        Self {
            id: format!("group-{index}"),
            index,
            name: name.into(),
            student_count,
            availability: availability.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_construction() {
        let fact = Group::new(0, "test", 32, vec![true; 40]);
        assert_eq!(fact.index, 0);
        assert_eq!(fact.id, "group-0");
        assert_eq!(fact.name, "test");
        assert_eq!(fact.student_count, 32);
        let _ = &fact.id;
        let _ = &fact.availability;
    }
}
