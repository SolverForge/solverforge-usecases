use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// One required seat on one flight. SolverForge chooses its employee.
#[planning_entity]
#[derive(Serialize, Deserialize)]
pub struct CrewAssignment {
    #[planning_id]
    pub id: String,
    pub flight_idx: usize,
    pub seat_number: u32,
    pub required_skill: String,
    pub departure_airport_idx: usize,
    pub arrival_airport_idx: usize,
    pub departure_minute: i64,
    pub arrival_minute: i64,
    #[serde(skip)]
    pub index: usize,
    // @solverforge:begin entity-variables
    #[planning_variable(
        value_range_provider = "employees",
        allows_unassigned = true,
        candidate_values = "qualified_employee_candidates",
        nearby_value_candidates = "qualified_employee_candidates"
    )]
    pub employee_idx: Option<usize>,
    // @solverforge:end entity-variables
}

pub(super) fn qualified_employee_candidates(
    solution: &crate::domain::Plan,
    entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    let Some(assignment) = solution.crew_assignments.get(entity_index) else {
        return &[];
    };
    if assignment.required_skill == "Pilot" {
        &solution.pilot_indices
    } else {
        &solution.attendant_indices
    }
}

impl CrewAssignment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        flight_idx: usize,
        seat_number: u32,
        required_skill: impl Into<String>,
        departure_airport_idx: usize,
        arrival_airport_idx: usize,
        departure_minute: i64,
        arrival_minute: i64,
    ) -> Self {
        Self {
            id: id.into(),
            flight_idx,
            seat_number,
            required_skill: required_skill.into(),
            departure_airport_idx,
            arrival_airport_idx,
            departure_minute,
            arrival_minute,
            index: 0,
            // @solverforge:begin entity-variable-init
            employee_idx: None,
            // @solverforge:end entity-variable-init
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crew_assignment_construction() {
        let entity = CrewAssignment::new("test-id", 0, 1, "Pilot", 0, 1, 60, 120);
        assert_eq!(entity.id, "test-id");
        let _ = &entity.flight_idx;
        let _ = &entity.seat_number;
        let _ = &entity.required_skill;
    }
}
