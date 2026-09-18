mod changeover;
mod constants;
mod enums;
mod support;
mod time;
mod workforce;

solverforge::planning_model! {
    root = "src/domain";

    mod plan;
    mod resources;
    mod furnace;
    mod coverage;

    pub use plan::Plan;
}

pub(crate) use self::coverage::{
    operator_shift_construction_entity_order_key, operator_shift_construction_value_order_key,
};
pub(crate) use self::coverage::{
    OperatorMonitoringCapacityEntries, OperatorNightWindowEntries, OperatorShiftAssignment,
    OperatorShiftCoverageEntries, OperatorTaskShiftWorkEntries,
};
pub(crate) use self::furnace::{
    furnace_assignment_construction_entity_order_key,
    furnace_assignment_construction_value_order_key,
    furnace_assignment_local_search_value_order_key,
    load_build_operator_construction_value_order_key,
    program_operator_construction_value_order_key, quench_operator_construction_value_order_key,
    task_operator_construction_entity_order_key, unload_operator_construction_value_order_key,
};
pub(crate) use self::furnace::{
    AssignedTaskCapacityEntries, AssignedTaskWithoutScheduleViolations, FurnaceAssignment,
    ManualTaskBookingEntries, MissingTaskOperatorViolations, MonitoringDemandEntries,
    TaskShiftWorkEntries,
};
pub(crate) use self::plan::PlanConstraintStreams;
pub(crate) use self::resources::{
    Furnace, Operator, Shift, ShiftCoverageDemand, ShiftCoverageEntries, WorkOrder,
};
pub(crate) use changeover::calculate_changeover_cost_for_sequence;
pub(crate) use constants::*;
pub(crate) use enums::*;
pub(crate) use support::*;
pub(crate) use time::*;
pub(crate) use workforce::{
    allowed_shift_types_for, base_role_skill_mask, has_insufficient_rest, requires_quench,
    shift_rest_minutes, MAX_CONSECUTIVE_NIGHTS, MAX_VISIBLE_SHIFTS_PER_WEEK, MINIMUM_REST_MINUTES,
    TARGET_VISIBLE_SHIFTS_PER_WEEK,
};
