use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// A teacher with a weekly availability calendar.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Teacher {
    #[planning_id]
    pub id: String,
    #[serde(skip)]
    pub index: usize, // the solver-facing join key
    pub name: String,
    pub availability: Vec<bool>,
}

impl Teacher {
    pub fn new(index: usize, name: impl Into<String>, availability: impl Into<Vec<bool>>) -> Self {
        Self {
            id: format!("teacher-{index}"),
            index,
            name: name.into(),
            availability: availability.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teacher_construction() {
        let fact = Teacher::new(0, "test", vec![true; 40]);
        assert_eq!(fact.index, 0);
        assert_eq!(fact.id, "teacher-0");
        assert_eq!(fact.name, "test");
        let _ = &fact.name;
        let _ = &fact.availability;
    }
}
