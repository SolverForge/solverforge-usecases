use super::{builders, scenario};
use crate::constraints::support::{
    apply_parts_delay, apply_readiness_floor, apply_technician_shortage,
};
use crate::domain::Plan;

pub fn baseline_solution_plan() -> Plan {
    let mut plan = scenario::baseline_plan();
    builders::assign_baseline_decisions(&mut plan);
    plan
}

pub fn improvable_solve_seed_plan() -> Plan {
    let mut plan = baseline_solution_plan();

    for (index, assignment) in plan.inspection_assignments.iter_mut().enumerate() {
        let shift = match index % 4 {
            0 => 2,
            1 | 2 => 1,
            _ => 0,
        };
        builders::shift_assignment_day(&mut assignment.day_idx, shift);
    }

    for (index, assignment) in plan.training_assignments.iter_mut().enumerate() {
        let shift = match index % 5 {
            0 => 2,
            1 | 3 => 1,
            _ => 0,
        };
        builders::shift_assignment_day(&mut assignment.day_idx, shift);
    }

    plan
}

pub fn technician_shortage_solution_plan() -> Plan {
    let mut plan = baseline_solution_plan();
    apply_technician_shortage(&mut plan, "ELEC", -1);
    plan
}

pub fn delayed_parts_plan() -> Plan {
    let mut plan = baseline_solution_plan();
    apply_parts_delay(&mut plan, "DELIV-01", 29);
    plan
}

pub fn raised_readiness_plan() -> Plan {
    let mut plan = baseline_solution_plan();
    apply_readiness_floor(&mut plan, 20);
    plan
}
