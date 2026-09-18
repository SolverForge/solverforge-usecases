use crate::domain::{
    allowed_shift_types_for, Operator, OperatorShiftAssignment, Plan, Shift,
    STAFFING_ROSTER_DAY_END, STAFFING_ROSTER_DAY_START,
};

use super::base_data::build_demo;
use super::catalog::{DemoData, DemoSeed};

#[cfg(test)]
pub(crate) fn build_demo_plan() -> Plan {
    build_demo_plan_for(DemoData::DEFAULT)
}

#[cfg(test)]
pub(crate) fn build_demo_plan_for(demo: DemoData) -> Plan {
    build_canonical_plan(build_demo(demo.profile()))
}

#[cfg(test)]
pub(crate) fn build_demo_problem() -> Plan {
    build_demo_problem_for(DemoData::DEFAULT)
}

pub(crate) fn build_demo_problem_for(demo: DemoData) -> Plan {
    build_canonical_plan(build_demo(demo.profile()))
}

fn build_canonical_plan(demo: DemoSeed) -> Plan {
    let operator_shift_assignments =
        build_operator_shift_assignments(&demo.operators, &demo.shifts);
    Plan {
        furnaces: demo.furnaces,
        work_orders: demo.work_orders,
        operators: demo.operators,
        shifts: demo.shifts,
        shift_coverage_demands: demo.shift_coverage_demands,
        assignments: demo.assignments,
        operator_shift_assignments,
        score: None,
    }
}

fn build_operator_shift_assignments(
    operators: &[Operator],
    _shifts: &[Shift],
) -> Vec<OperatorShiftAssignment> {
    let mut assignments = Vec::with_capacity(operators.len() * 8);
    let mut assignment_id = 0usize;

    for operator in operators {
        for roster_day in STAFFING_ROSTER_DAY_START..=STAFFING_ROSTER_DAY_END {
            assignments.push(OperatorShiftAssignment {
                id: assignment_id,
                operator_idx: operator.id,
                operator_name: operator.name,
                role: operator.role,
                day_only: operator.day_only,
                roster_day,
                skill_mask: operator.skill_mask,
                allowed_shift_types: allowed_shift_types_for(
                    operator.role,
                    operator.day_only,
                    roster_day,
                ),
                shift_type: None,
            });
            assignment_id += 1;
        }
    }

    assignments
}
