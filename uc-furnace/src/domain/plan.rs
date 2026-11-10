use solverforge::prelude::*;

// @solverforge:begin solution-imports
use super::Furnace;
use super::FurnaceAssignment;
use super::Operator;
use super::OperatorShiftAssignment;
use super::Shift;
use super::ShiftCoverageDemand;
use super::WorkOrder;
// @solverforge:end solution-imports
use super::ShiftId;

#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml",
    scalar_groups = "scalar_groups",
    conflict_repairs = "conflict_repairs"
)]
pub struct Plan {
    // @solverforge:begin solution-collections
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
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
}

impl Plan {
    /// Builds an unsolved plan from its facts and planning entities.
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        furnaces: Vec<Furnace>,
        work_orders: Vec<WorkOrder>,
        operators: Vec<Operator>,
        shifts: Vec<Shift>,
        shift_coverage_demands: Vec<ShiftCoverageDemand>,
        assignments: Vec<FurnaceAssignment>,
        operator_shift_assignments: Vec<OperatorShiftAssignment>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        Self {
            // @solverforge:begin solution-constructor-init
            furnaces,
            work_orders,
            operators,
            shifts,
            shift_coverage_demands,
            assignments,
            operator_shift_assignments,
            // @solverforge:end solution-constructor-init
            score: None,
        }
    }

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

// @solverforge:begin scalar-groups
pub(super) fn scalar_groups() -> Vec<ScalarGroup<Plan>> {
    vec![
        ScalarGroup::candidates(
            "furnace_assignment",
            vec![Plan::assignments().scalar("assignment")],
            crate::solver::scalar_groups::furnace_assignment_candidates,
        ),
        ScalarGroup::candidates(
            "roster_shift_assignment",
            vec![Plan::operator_shift_assignments().scalar("shift_type")],
            crate::solver::scalar_groups::roster_shift_assignment_candidates,
        ),
        ScalarGroup::candidates(
            "task_operator_assignment",
            vec![
                Plan::assignments().scalar("load_build_operator_id"),
                Plan::assignments().scalar("program_operator_id"),
                Plan::assignments().scalar("quench_operator_id"),
                Plan::assignments().scalar("unload_operator_id"),
            ],
            crate::solver::scalar_groups::task_operator_assignment_candidates,
        ),
        ScalarGroup::candidates(
            "furnace_schedule_task_assignment",
            vec![
                Plan::assignments().scalar("assignment"),
                Plan::assignments().scalar("load_build_operator_id"),
                Plan::assignments().scalar("program_operator_id"),
                Plan::assignments().scalar("quench_operator_id"),
                Plan::assignments().scalar("unload_operator_id"),
            ],
            crate::solver::scalar_groups::furnace_schedule_task_assignment_candidates,
        ),
    ]
}
// @solverforge:end scalar-groups

// @solverforge:begin conflict-repairs
pub(super) fn conflict_repairs() -> Vec<ConflictRepair<Plan>> {
    vec![
        ConflictRepair::new(
            "minimum_rest",
            crate::solver::conflict_repair::minimum_rest_repairs,
        ),
        ConflictRepair::new(
            "visible_shift_limit",
            crate::solver::conflict_repair::visible_shift_limit_repairs,
        ),
        ConflictRepair::new(
            "task_operator_on_owning_shift",
            crate::solver::conflict_repair::task_owning_shift_repairs,
        ),
        ConflictRepair::new(
            "shift_role_coverage",
            crate::solver::conflict_repair::shift_role_coverage_repairs,
        ),
        ConflictRepair::new(
            "monitoring_capacity",
            crate::solver::conflict_repair::monitoring_capacity_repairs,
        ),
        ConflictRepair::new(
            "furnace_overlap",
            crate::solver::conflict_repair::furnace_overlap_repairs,
        ),
        ConflictRepair::new(
            "changeover_gap",
            crate::solver::conflict_repair::changeover_gap_repairs,
        ),
    ]
}
// @solverforge:end conflict-repairs
