use solverforge::prelude::*;

use super::{
    Furnace, FurnaceAssignment, Operator, OperatorShiftAssignment, Shift, ShiftCoverageDemand,
    ShiftId, WorkOrder,
};

#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml",
    conflict_repairs = "crate::solver::conflict_repair::providers",
    scalar_groups = "crate::solver::scalar_groups::groups"
)]
pub struct Plan {
    #[problem_fact_collection]
    pub furnaces: Vec<Furnace>,
    #[problem_fact_collection]
    pub work_orders: Vec<WorkOrder>,
    #[problem_fact_collection]
    pub operators: Vec<Operator>,
    #[problem_fact_collection]
    pub shifts: Vec<Shift>,
    #[problem_fact_collection]
    pub shift_coverage_demands: Vec<ShiftCoverageDemand>,
    #[planning_entity_collection]
    pub assignments: Vec<FurnaceAssignment>,
    #[planning_entity_collection]
    pub operator_shift_assignments: Vec<OperatorShiftAssignment>,
    #[planning_score]
    pub score: Option<HardSoftScore>,
}

impl Plan {
    pub fn furnace(&self, idx: usize) -> &Furnace {
        &self.furnaces[idx]
    }

    pub fn work_order(&self, idx: usize) -> &WorkOrder {
        &self.work_orders[idx]
    }

    pub fn operator(&self, idx: usize) -> &Operator {
        &self.operators[idx]
    }

    pub fn shift_by_id(&self, shift_id: ShiftId) -> Option<&Shift> {
        self.shifts
            .get(shift_id)
            .filter(|shift| shift.id == shift_id)
    }
}
