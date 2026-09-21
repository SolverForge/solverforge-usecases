use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::Airport;
use super::Employee;
use super::Flight;
use super::CrewAssignment;
// @solverforge:end solution-imports

/// Complete flight-crew planning problem and its current score.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[derive(Serialize, Deserialize)]
pub struct Plan {
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub airports: Vec<Airport>,
    #[problem_fact_collection]
    pub employees: Vec<Employee>,
    #[problem_fact_collection]
    pub flights: Vec<Flight>,
    #[planning_entity_collection]
    pub crew_assignments: Vec<CrewAssignment>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
    #[serde(skip)]
    pub(crate) pilot_indices: Vec<usize>,
    #[serde(skip)]
    pub(crate) attendant_indices: Vec<usize>,
}

impl Plan {
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        airports: Vec<Airport>,
        employees: Vec<Employee>,
        flights: Vec<Flight>,
        crew_assignments: Vec<CrewAssignment>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        let mut plan = Self {
            // @solverforge:begin solution-constructor-init
            airports,
            employees,
            flights,
            crew_assignments,
            // @solverforge:end solution-constructor-init
            score: None,
            pilot_indices: Vec::new(),
            attendant_indices: Vec::new(),
        };
        plan.rebuild_derived_fields();
        plan
    }

    /// Rebuilds dense solver join keys and rejects stale assignment indexes.
    pub fn rebuild_derived_fields(&mut self) {
        for (index, airport) in self.airports.iter_mut().enumerate() {
            airport.index = index;
        }
        for (index, employee) in self.employees.iter_mut().enumerate() {
            employee.index = index;
        }
        self.pilot_indices = self.employees.iter().enumerate()
            .filter_map(|(index, employee)| employee.skills.contains(&"Pilot".to_string()).then_some(index))
            .collect();
        self.attendant_indices = self.employees.iter().enumerate()
            .filter_map(|(index, employee)| employee.skills.contains(&"Flight attendant".to_string()).then_some(index))
            .collect();
        for (index, flight) in self.flights.iter_mut().enumerate() {
            flight.index = index;
        }
        for (index, assignment) in self.crew_assignments.iter_mut().enumerate() {
            assignment.index = index;
            assignment.employee_idx = assignment
                .employee_idx
                .filter(|employee_idx| *employee_idx < self.employees.len());
        }
    }
}
