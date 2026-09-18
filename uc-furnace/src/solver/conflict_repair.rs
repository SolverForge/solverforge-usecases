use std::collections::{HashMap, HashSet};

use solverforge::prelude::*;

use crate::domain::{
    calculate_changeover_cost_for_sequence, heating_ramp_minutes, monitoring_bucket_for_minute,
    owning_shift_identity_for_interval, owning_shift_identity_for_minute, FurnaceAssignment,
    ManualTaskKind, MonitoringLoad, OperatorSkill, Plan, ShiftIdentity, ShiftType, HORIZON_MINUTES,
    NUM_TIME_SLOTS, OFF_SHIFT_VALUE, TIME_STEP,
};

pub(crate) fn providers() -> Vec<ConflictRepair<Plan>> {
    vec![
        ConflictRepair::new("minimumRest", minimum_rest_repairs),
        ConflictRepair::new("visibleShiftLimit", visible_shift_limit_repairs),
        ConflictRepair::new("taskOperatorOnOwningShift", task_owning_shift_repairs),
        ConflictRepair::new("shiftRoleCoverage", shift_role_coverage_repairs),
        ConflictRepair::new("monitoringCapacity", monitoring_capacity_repairs),
        ConflictRepair::new("furnaceOverlap", furnace_overlap_repairs),
        ConflictRepair::new("changeoverGap", changeover_gap_repairs),
    ]
}

fn minimum_rest_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut specs = Vec::new();

    for previous_index in 0..plan.operator_shift_assignments.len() {
        for current_index in 0..plan.operator_shift_assignments.len() {
            if specs.len() >= limits.max_moves_per_step {
                return specs;
            }
            let previous = &plan.operator_shift_assignments[previous_index];
            let current = &plan.operator_shift_assignments[current_index];
            if previous.operator_idx != current.operator_idx
                || !crate::domain::has_insufficient_rest(previous, current)
            {
                continue;
            }

            push_roster_relief_repairs(plan, &mut specs, limits, current_index, "minimumRest");
            push_roster_relief_repairs(plan, &mut specs, limits, previous_index, "minimumRest");
        }
    }

    specs
}

fn visible_shift_limit_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut by_operator: HashMap<usize, Vec<usize>> = HashMap::new();
    for (index, assignment) in plan.operator_shift_assignments.iter().enumerate() {
        if assignment.roster_day >= 0 && assignment.is_working() {
            by_operator
                .entry(assignment.operator_idx)
                .or_default()
                .push(index);
        }
    }

    let mut specs = Vec::new();
    for indices in by_operator.values_mut() {
        if indices.len() <= crate::domain::MAX_VISIBLE_SHIFTS_PER_WEEK {
            continue;
        }
        indices.sort_by_key(|idx| {
            let assignment = &plan.operator_shift_assignments[*idx];
            (
                assignment.roster_day,
                assignment.shift_type.unwrap_or(OFF_SHIFT_VALUE),
            )
        });

        for index in indices
            .iter()
            .rev()
            .take(indices.len() - crate::domain::MAX_VISIBLE_SHIFTS_PER_WEEK)
        {
            push_roster_removal_repairs(plan, &mut specs, limits, *index, "visibleShiftLimit");
        }
    }

    specs
}

fn task_owning_shift_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut specs = Vec::new();

    for (assignment_index, assignment) in plan.assignments.iter().enumerate() {
        for kind in assignment.required_task_kinds() {
            if specs.len() >= limits.max_moves_per_step {
                return specs;
            }
            let Some(shift) = assignment.task_shift_identity(kind) else {
                continue;
            };
            let Some(operator_id) = assignment.task_operator_id(kind) else {
                continue;
            };
            if operator_covers_shift(plan, operator_id, shift) {
                continue;
            }

            if let Some(roster_index) =
                roster_index_for_operator_day(plan, operator_id, shift.roster_day)
            {
                let roster_assignment = &plan.operator_shift_assignments[roster_index];
                if roster_assignment
                    .allowed_shift_types
                    .contains(&shift.shift_type.index())
                    && roster_value_is_legal(plan, roster_index, Some(shift.shift_type.index()))
                {
                    specs.push(RepairCandidate::new(
                        "taskOperatorOnOwningShift",
                        vec![Plan::operator_shift_assignments()
                            .scalar("shift_type")
                            .set(roster_index, Some(shift.shift_type.index()))],
                    ));
                }
            }

            let variable_name = task_variable_name(kind);
            for candidate_operator in assignment.task_eligible_operator_ids(kind) {
                if specs.len() >= limits.max_moves_per_step {
                    return specs;
                }
                if *candidate_operator == operator_id {
                    continue;
                }
                if operator_covers_shift(plan, *candidate_operator, shift) {
                    specs.push(RepairCandidate::new(
                        "taskOperatorOnOwningShift",
                        vec![Plan::assignments()
                            .scalar(variable_name)
                            .set(assignment_index, Some(*candidate_operator))],
                    ));
                    break;
                }
            }
        }
    }

    specs
}

fn shift_role_coverage_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut specs = Vec::new();

    for demand in &plan.shift_coverage_demands {
        if specs.len() >= limits.max_moves_per_step {
            return specs;
        }
        let Some(shift) = plan
            .shift_by_id(demand.shift_id)
            .map(|shift| shift.identity())
        else {
            continue;
        };
        let assigned = plan
            .operator_shift_assignments
            .iter()
            .filter(|assignment| {
                assignment.role == demand.role && assignment.shift_id() == Some(demand.shift_id)
            })
            .count() as i64;
        let shortage = demand.required_count.saturating_sub(assigned);
        if shortage <= 0 {
            continue;
        }

        let mut emitted = 0_i64;
        for (roster_index, assignment) in plan.operator_shift_assignments.iter().enumerate() {
            if emitted >= shortage || specs.len() >= limits.max_moves_per_step {
                break;
            }
            if assignment.role != demand.role
                || assignment.roster_day != shift.roster_day
                || assignment.shift_type == Some(shift.shift_type.index())
                || !assignment
                    .allowed_shift_types
                    .contains(&shift.shift_type.index())
                || !roster_value_is_legal(plan, roster_index, Some(shift.shift_type.index()))
            {
                continue;
            }

            specs.push(RepairCandidate::new(
                "shiftRoleCoverage",
                vec![Plan::operator_shift_assignments()
                    .scalar("shift_type")
                    .set(roster_index, Some(shift.shift_type.index()))],
            ));
            emitted += 1;
        }
    }

    specs
}

fn monitoring_capacity_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut specs = Vec::new();
    let mut loads = monitoring_loads(plan);
    let mut repaired_shifts = HashSet::new();

    for (bucket, load) in loads.drain() {
        if specs.len() >= limits.max_moves_per_step {
            return specs;
        }
        if load.required_operators() <= load.capacity {
            continue;
        }
        let minute = bucket * TIME_STEP;
        let Some(shift) = owning_shift_identity_for_minute(minute) else {
            continue;
        };
        if !repaired_shifts.insert(shift.shift_id) {
            continue;
        }

        for (roster_index, assignment) in plan.operator_shift_assignments.iter().enumerate() {
            if specs.len() >= limits.max_moves_per_step {
                return specs;
            }
            if assignment.roster_day != shift.roster_day
                || !assignment.skill_mask.contains(OperatorSkill::Monitor)
                || assignment.shift_type == Some(shift.shift_type.index())
                || !assignment
                    .allowed_shift_types
                    .contains(&shift.shift_type.index())
                || !roster_value_is_legal(plan, roster_index, Some(shift.shift_type.index()))
            {
                continue;
            }

            specs.push(RepairCandidate::new(
                "monitoringCapacity",
                vec![Plan::operator_shift_assignments()
                    .scalar("shift_type")
                    .set(roster_index, Some(shift.shift_type.index()))],
            ));
            break;
        }
    }

    specs
}

fn furnace_overlap_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut specs = Vec::new();

    for left_index in 0..plan.assignments.len() {
        for right_index in (left_index + 1)..plan.assignments.len() {
            if specs.len() >= limits.max_moves_per_step {
                return specs;
            }
            let left = &plan.assignments[left_index];
            let right = &plan.assignments[right_index];
            if left.furnace_idx().is_none()
                || left.furnace_idx() != right.furnace_idx()
                || !scheduled_assignments_overlap(left, right)
            {
                continue;
            }

            if let Some(value) = first_non_conflicting_assignment_value(plan, right_index) {
                specs.push(RepairCandidate::new(
                    "furnaceOverlap",
                    vec![Plan::assignments()
                        .scalar("assignment")
                        .set(right_index, Some(value))],
                ));
            }
            if specs.len() >= limits.max_moves_per_step {
                return specs;
            }
            if let Some(value) = first_non_conflicting_assignment_value(plan, left_index) {
                specs.push(RepairCandidate::new(
                    "furnaceOverlap",
                    vec![Plan::assignments()
                        .scalar("assignment")
                        .set(left_index, Some(value))],
                ));
            }
        }
    }

    specs
}

fn changeover_gap_repairs(plan: &Plan, limits: RepairLimits) -> Vec<RepairCandidate<Plan>> {
    let mut specs = Vec::new();
    let mut matches_seen = 0usize;

    for left_index in 0..plan.assignments.len() {
        for right_index in 0..plan.assignments.len() {
            if specs.len() >= limits.max_moves_per_step
                || matches_seen >= limits.max_matches_per_step
            {
                return specs;
            }
            if left_index == right_index {
                continue;
            }
            let left = &plan.assignments[left_index];
            let right = &plan.assignments[right_index];
            if !scheduled_assignments_have_changeover_gap(left, right) {
                continue;
            }
            matches_seen += 1;

            push_first_assignment_value_repair(plan, &mut specs, right_index, "changeoverGap");
            if specs.len() >= limits.max_moves_per_step {
                return specs;
            }
            push_first_assignment_value_repair(plan, &mut specs, left_index, "changeoverGap");
        }
    }

    specs
}

fn push_roster_relief_repairs(
    plan: &Plan,
    specs: &mut Vec<RepairCandidate<Plan>>,
    limits: RepairLimits,
    roster_index: usize,
    reason: &'static str,
) {
    let assignment = &plan.operator_shift_assignments[roster_index];
    push_roster_removal_repairs(plan, specs, limits, roster_index, reason);

    for value in assignment
        .allowed_shift_types
        .iter()
        .copied()
        .filter(|value| Some(*value) != assignment.shift_type)
    {
        if specs.len() >= limits.max_moves_per_step {
            return;
        }
        if value == OFF_SHIFT_VALUE {
            continue;
        }
        if !assignment.allowed_shift_types.contains(&value) {
            continue;
        }
        push_roster_value_change_repairs(plan, specs, limits, roster_index, value, reason);
    }
}

fn push_roster_value_change_repairs(
    plan: &Plan,
    specs: &mut Vec<RepairCandidate<Plan>>,
    limits: RepairLimits,
    roster_index: usize,
    value: usize,
    reason: &'static str,
) {
    if specs.len() >= limits.max_moves_per_step {
        return;
    }
    let assignment = &plan.operator_shift_assignments[roster_index];
    if !assignment.is_working() || shift_type_from_value(Some(value)).is_none() {
        return;
    }
    let change_edit = Plan::operator_shift_assignments()
        .scalar("shift_type")
        .set(roster_index, Some(value));
    if !roster_edits_are_legal(plan, std::slice::from_ref(&change_edit)) {
        return;
    }

    let needs = removal_replacement_needs(plan, roster_index);
    if !needs.requires_replacement() {
        specs.push(RepairCandidate::new(reason, vec![change_edit]));
        return;
    }
    if !needs.requires_roster_replacement()
        && push_task_reassignment_only_repairs(
            plan,
            specs,
            limits,
            reason,
            change_edit,
            &needs,
            assignment.operator_idx,
        )
    {
        return;
    }
    for replacement_index in replacement_roster_indices(plan, roster_index, &needs) {
        if specs.len() >= limits.max_moves_per_step {
            return;
        }
        let replacement_operator = plan.operator_shift_assignments[replacement_index].operator_idx;
        let mut edits = vec![
            change_edit,
            Plan::operator_shift_assignments()
                .scalar("shift_type")
                .set(replacement_index, Some(needs.shift.shift_type.index())),
        ];
        append_task_reassignment_edits(
            plan,
            &needs.task_bookings,
            replacement_operator,
            &mut edits,
        );
        if roster_edits_are_legal(plan, &edits)
            && task_reassignments_are_clean(plan, &needs.task_bookings, replacement_operator)
        {
            specs.push(RepairCandidate::new(reason, edits));
        }
    }
}

fn push_roster_removal_repairs(
    plan: &Plan,
    specs: &mut Vec<RepairCandidate<Plan>>,
    limits: RepairLimits,
    roster_index: usize,
    reason: &'static str,
) {
    if specs.len() >= limits.max_moves_per_step {
        return;
    }
    let assignment = &plan.operator_shift_assignments[roster_index];
    if !assignment.is_working()
        || assignment.shift_type == Some(OFF_SHIFT_VALUE)
        || !assignment.allowed_shift_types.contains(&OFF_SHIFT_VALUE)
    {
        return;
    }

    let off_edit = Plan::operator_shift_assignments()
        .scalar("shift_type")
        .set(roster_index, Some(OFF_SHIFT_VALUE));
    if !roster_edits_are_legal(plan, std::slice::from_ref(&off_edit)) {
        return;
    }

    let needs = removal_replacement_needs(plan, roster_index);
    if !needs.requires_replacement() {
        specs.push(RepairCandidate::new(reason, vec![off_edit]));
        return;
    }
    if !needs.requires_roster_replacement()
        && push_task_reassignment_only_repairs(
            plan,
            specs,
            limits,
            reason,
            off_edit,
            &needs,
            assignment.operator_idx,
        )
    {
        return;
    }
    for replacement_index in replacement_roster_indices(plan, roster_index, &needs) {
        if specs.len() >= limits.max_moves_per_step {
            return;
        }
        let replacement_operator = plan.operator_shift_assignments[replacement_index].operator_idx;
        let mut edits = vec![
            off_edit,
            Plan::operator_shift_assignments()
                .scalar("shift_type")
                .set(replacement_index, Some(needs.shift.shift_type.index())),
        ];
        append_task_reassignment_edits(
            plan,
            &needs.task_bookings,
            replacement_operator,
            &mut edits,
        );
        if roster_edits_are_legal(plan, &edits)
            && task_reassignments_are_clean(plan, &needs.task_bookings, replacement_operator)
        {
            specs.push(RepairCandidate::new(reason, edits));
        }
    }
}

struct RemovalReplacementNeeds {
    shift: ShiftIdentity,
    role_coverage: bool,
    monitoring_capacity: bool,
    task_bookings: Vec<TaskBookingNeed>,
}

impl RemovalReplacementNeeds {
    fn requires_replacement(&self) -> bool {
        self.role_coverage || self.monitoring_capacity || !self.task_bookings.is_empty()
    }

    fn requires_roster_replacement(&self) -> bool {
        self.role_coverage || self.monitoring_capacity
    }
}

#[derive(Clone, Copy)]
struct TaskBookingNeed {
    assignment_index: usize,
    kind: ManualTaskKind,
    start: usize,
    end: usize,
}

fn removal_replacement_needs(plan: &Plan, roster_index: usize) -> RemovalReplacementNeeds {
    let assignment = &plan.operator_shift_assignments[roster_index];
    let shift = assignment
        .shift_id()
        .and_then(|shift_id| plan.shift_by_id(shift_id).map(|shift| shift.identity()))
        .expect("working roster assignment must have a shift identity");
    let task_bookings = task_bookings_for_operator_shift(plan, assignment.operator_idx, shift);
    RemovalReplacementNeeds {
        shift,
        role_coverage: removal_would_break_role_coverage(plan, roster_index, shift),
        monitoring_capacity: assignment.skill_mask.contains(OperatorSkill::Monitor)
            && removal_would_break_monitoring_capacity(plan, roster_index),
        task_bookings,
    }
}

fn replacement_roster_indices(
    plan: &Plan,
    removed_index: usize,
    needs: &RemovalReplacementNeeds,
) -> Vec<usize> {
    let removed = &plan.operator_shift_assignments[removed_index];
    let mut candidates = Vec::new();
    for (index, assignment) in plan.operator_shift_assignments.iter().enumerate() {
        if index == removed_index
            || assignment.roster_day != needs.shift.roster_day
            || assignment.operator_idx == removed.operator_idx
            || !assignment
                .allowed_shift_types
                .contains(&needs.shift.shift_type.index())
            || assignment
                .shift_type
                .is_some_and(|value| value != OFF_SHIFT_VALUE)
        {
            continue;
        }
        if needs.role_coverage && assignment.role != removed.role {
            continue;
        }
        if needs.monitoring_capacity && !assignment.skill_mask.contains(OperatorSkill::Monitor) {
            continue;
        }
        if !task_reassignments_are_clean(plan, &needs.task_bookings, assignment.operator_idx) {
            continue;
        }
        candidates.push(index);
    }
    candidates.sort_by_key(|index| {
        let assignment = &plan.operator_shift_assignments[*index];
        (
            visible_shift_count_with_edits(plan, assignment.operator_idx, &[]),
            night_count_with_edits(plan, assignment.operator_idx, &[]),
            assignment.operator_idx,
        )
    });
    candidates
}

fn push_task_reassignment_only_repairs(
    plan: &Plan,
    specs: &mut Vec<RepairCandidate<Plan>>,
    limits: RepairLimits,
    reason: &'static str,
    roster_edit: ScalarEdit<Plan>,
    needs: &RemovalReplacementNeeds,
    removed_operator: usize,
) -> bool {
    let mut emitted = false;
    for replacement_operator in task_reassignment_operator_indices(
        plan,
        &needs.task_bookings,
        needs.shift,
        removed_operator,
    ) {
        if specs.len() >= limits.max_moves_per_step {
            return emitted;
        }
        let mut edits = vec![roster_edit];
        append_task_reassignment_edits(
            plan,
            &needs.task_bookings,
            replacement_operator,
            &mut edits,
        );
        specs.push(RepairCandidate::new(reason, edits));
        emitted = true;
    }
    emitted
}

fn task_reassignment_operator_indices(
    plan: &Plan,
    task_bookings: &[TaskBookingNeed],
    shift: ShiftIdentity,
    removed_operator: usize,
) -> Vec<usize> {
    let mut operators = Vec::new();
    for assignment in &plan.operator_shift_assignments {
        let operator = assignment.operator_idx;
        if operator == removed_operator
            || operators.contains(&operator)
            || assignment.shift_id() != Some(shift.shift_id)
            || !task_reassignments_are_clean(plan, task_bookings, operator)
        {
            continue;
        }
        operators.push(operator);
    }
    operators.sort_by_key(|operator| {
        (
            visible_shift_count_with_edits(plan, *operator, &[]),
            night_count_with_edits(plan, *operator, &[]),
            *operator,
        )
    });
    operators
}

fn append_task_reassignment_edits(
    plan: &Plan,
    task_bookings: &[TaskBookingNeed],
    replacement_operator: usize,
    edits: &mut Vec<ScalarEdit<Plan>>,
) {
    for booking in task_bookings {
        let assignment = &plan.assignments[booking.assignment_index];
        if assignment.task_operator_id(booking.kind) == Some(replacement_operator) {
            continue;
        }
        edits.push(
            Plan::assignments()
                .scalar(task_variable_name(booking.kind))
                .set(booking.assignment_index, Some(replacement_operator)),
        );
    }
}

fn removal_would_break_role_coverage(
    plan: &Plan,
    roster_index: usize,
    shift: ShiftIdentity,
) -> bool {
    let assignment = &plan.operator_shift_assignments[roster_index];
    let required = plan
        .shift_coverage_demands
        .iter()
        .filter(|demand| demand.shift_id == shift.shift_id && demand.role == assignment.role)
        .map(|demand| demand.required_count)
        .sum::<i64>();
    if required <= 0 {
        return false;
    }
    let assigned = plan
        .operator_shift_assignments
        .iter()
        .enumerate()
        .filter(|(index, other)| {
            *index != roster_index
                && other.role == assignment.role
                && other.shift_id() == Some(shift.shift_id)
        })
        .count() as i64;
    assigned < required
}

fn removal_would_break_monitoring_capacity(plan: &Plan, roster_index: usize) -> bool {
    monitoring_loads_with_roster_edits(
        plan,
        &[RosterValueEdit {
            index: roster_index,
            value: Some(OFF_SHIFT_VALUE),
        }],
    )
    .values()
    .any(|load| load.required_operators() > load.capacity)
}

fn task_bookings_for_operator_shift(
    plan: &Plan,
    operator_id: usize,
    shift: ShiftIdentity,
) -> Vec<TaskBookingNeed> {
    let mut bookings = Vec::new();
    for (assignment_index, assignment) in plan.assignments.iter().enumerate() {
        for kind in assignment.required_task_kinds() {
            if assignment.task_operator_id(kind) != Some(operator_id)
                || assignment.task_shift_identity(kind) != Some(shift)
            {
                continue;
            }
            let Some((start, end)) = assignment.task_interval(kind) else {
                continue;
            };
            bookings.push(TaskBookingNeed {
                assignment_index,
                kind,
                start,
                end,
            });
        }
    }
    bookings
}

fn task_reassignments_are_clean(
    plan: &Plan,
    task_bookings: &[TaskBookingNeed],
    replacement_operator: usize,
) -> bool {
    if task_bookings.is_empty() {
        return true;
    }
    for booking in task_bookings {
        let assignment = &plan.assignments[booking.assignment_index];
        if !assignment
            .task_eligible_operator_ids(booking.kind)
            .contains(&replacement_operator)
        {
            return false;
        }
        let Some(operator) = plan
            .operators
            .iter()
            .find(|operator| operator.id == replacement_operator)
        else {
            return false;
        };
        if !operator
            .skill_mask
            .contains_all(assignment.task_required_skills(booking.kind))
        {
            return false;
        }
    }

    let mut intervals = Vec::new();
    for assignment in &plan.assignments {
        assignment.for_each_manual_task_booking(|existing| {
            if existing.operator_id == replacement_operator
                && !task_bookings.iter().any(|booking| {
                    plan.assignments[booking.assignment_index].work_order_idx
                        == existing.work_order_idx
                        && booking.kind == existing.kind
                })
            {
                intervals.push((existing.start_minute, existing.end_minute));
            }
        });
    }
    for booking in task_bookings {
        if intervals
            .iter()
            .any(|&(start, end)| booking.start < end && start < booking.end)
        {
            return false;
        }
        intervals.push((booking.start, booking.end));
    }
    true
}

fn roster_value_is_legal(plan: &Plan, roster_index: usize, value: Option<usize>) -> bool {
    roster_edits_are_legal(
        plan,
        &[Plan::operator_shift_assignments()
            .scalar("shift_type")
            .set(roster_index, value)],
    )
}

#[derive(Clone, Copy)]
struct RosterValueEdit {
    index: usize,
    value: Option<usize>,
}

fn roster_edits_are_legal(plan: &Plan, edits: &[ScalarEdit<Plan>]) -> bool {
    edits
        .iter()
        .filter(|edit| edit.variable_name() == "shift_type")
        .all(|edit| {
            let Some(assignment) = plan.operator_shift_assignments.get(edit.entity_index()) else {
                return false;
            };
            if !edit
                .to_value()
                .is_none_or(|value| assignment.allowed_shift_types.contains(&value))
            {
                return false;
            }
            !(assignment.day_only
                && shift_type_from_value(edit.to_value()) == Some(ShiftType::Night))
        })
}

fn roster_shift_type_with_edits(
    index: usize,
    current: Option<usize>,
    edits: &[RosterValueEdit],
) -> Option<ShiftType> {
    let value = edits
        .iter()
        .rev()
        .find(|edit| edit.index == index)
        .map_or(current, |edit| edit.value);
    shift_type_from_value(value)
}

fn shift_type_from_value(value: Option<usize>) -> Option<ShiftType> {
    match value {
        Some(0) => Some(ShiftType::Morning),
        Some(1) => Some(ShiftType::Afternoon),
        Some(2) => Some(ShiftType::Night),
        _ => None,
    }
}

fn visible_shift_count_with_edits(
    plan: &Plan,
    operator_id: usize,
    edits: &[RosterValueEdit],
) -> usize {
    plan.operator_shift_assignments
        .iter()
        .enumerate()
        .filter(|(index, assignment)| {
            assignment.operator_idx == operator_id
                && assignment.roster_day >= 0
                && roster_shift_type_with_edits(*index, assignment.shift_type, edits).is_some()
        })
        .count()
}

fn night_count_with_edits(plan: &Plan, operator_id: usize, edits: &[RosterValueEdit]) -> usize {
    plan.operator_shift_assignments
        .iter()
        .enumerate()
        .filter(|(index, assignment)| {
            assignment.operator_idx == operator_id
                && roster_shift_type_with_edits(*index, assignment.shift_type, edits)
                    == Some(ShiftType::Night)
        })
        .count()
}

fn monitoring_loads_with_roster_edits(
    plan: &Plan,
    edits: &[RosterValueEdit],
) -> HashMap<usize, MonitoringLoad> {
    let mut loads: HashMap<usize, MonitoringLoad> = HashMap::new();

    for assignment in &plan.assignments {
        let Some(start) = assignment.start_minutes() else {
            continue;
        };
        let Some(end) = assignment.end_minutes() else {
            continue;
        };
        let ramp_end = start + heating_ramp_minutes(assignment.duration_minutes());
        add_monitoring_load(
            &mut loads,
            start,
            ramp_end,
            MonitoringLoad {
                ramp: 1,
                ..MonitoringLoad::default()
            },
        );
        add_monitoring_load(
            &mut loads,
            ramp_end,
            end,
            MonitoringLoad {
                soak: 1,
                ..MonitoringLoad::default()
            },
        );
        assignment.for_each_manual_task_booking(|booking| {
            add_monitoring_load(
                &mut loads,
                booking.start_minute,
                booking.end_minute,
                MonitoringLoad {
                    task: 1,
                    ..MonitoringLoad::default()
                },
            );
        });
    }

    for (index, assignment) in plan.operator_shift_assignments.iter().enumerate() {
        if !assignment.skill_mask.contains(OperatorSkill::Monitor) {
            continue;
        }
        let Some(shift_type) = roster_shift_type_with_edits(index, assignment.shift_type, edits)
        else {
            continue;
        };
        let Some((start, end)) = crate::domain::shift_bounds(assignment.roster_day, shift_type)
        else {
            continue;
        };
        add_monitoring_load(
            &mut loads,
            start,
            end,
            MonitoringLoad {
                capacity: 1,
                ..MonitoringLoad::default()
            },
        );
    }

    loads
}

fn operator_covers_shift(plan: &Plan, operator_id: usize, shift: ShiftIdentity) -> bool {
    plan.operator_shift_assignments.iter().any(|assignment| {
        assignment.operator_idx == operator_id && assignment.shift_id() == Some(shift.shift_id)
    })
}

fn roster_index_for_operator_day(
    plan: &Plan,
    operator_id: usize,
    roster_day: i32,
) -> Option<usize> {
    plan.operator_shift_assignments
        .iter()
        .position(|assignment| {
            assignment.operator_idx == operator_id && assignment.roster_day == roster_day
        })
}

fn task_variable_name(kind: ManualTaskKind) -> &'static str {
    match kind {
        ManualTaskKind::LoadBuild => "load_build_operator_id",
        ManualTaskKind::Program => "program_operator_id",
        ManualTaskKind::Quench => "quench_operator_id",
        ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => "unload_operator_id",
    }
}

fn monitoring_loads(plan: &Plan) -> HashMap<usize, MonitoringLoad> {
    let mut loads: HashMap<usize, MonitoringLoad> = HashMap::new();

    for assignment in &plan.assignments {
        let Some(start) = assignment.start_minutes() else {
            continue;
        };
        let Some(end) = assignment.end_minutes() else {
            continue;
        };
        let ramp_end = start + heating_ramp_minutes(assignment.duration_minutes());
        add_monitoring_load(
            &mut loads,
            start,
            ramp_end,
            MonitoringLoad {
                ramp: 1,
                ..MonitoringLoad::default()
            },
        );
        add_monitoring_load(
            &mut loads,
            ramp_end,
            end,
            MonitoringLoad {
                soak: 1,
                ..MonitoringLoad::default()
            },
        );
        assignment.for_each_manual_task_booking(|booking| {
            add_monitoring_load(
                &mut loads,
                booking.start_minute,
                booking.end_minute,
                MonitoringLoad {
                    task: 1,
                    ..MonitoringLoad::default()
                },
            );
        });
    }

    for assignment in &plan.operator_shift_assignments {
        if !assignment.is_working() || !assignment.skill_mask.contains(OperatorSkill::Monitor) {
            continue;
        }
        let Some(shift_type) = assignment.shift_type_enum() else {
            continue;
        };
        let Some((start, end)) = crate::domain::shift_bounds(assignment.roster_day, shift_type)
        else {
            continue;
        };
        add_monitoring_load(
            &mut loads,
            start,
            end,
            MonitoringLoad {
                capacity: 1,
                ..MonitoringLoad::default()
            },
        );
    }

    loads
}

fn add_monitoring_load(
    loads: &mut HashMap<usize, MonitoringLoad>,
    start_minute: usize,
    end_minute: usize,
    load: MonitoringLoad,
) {
    if end_minute <= start_minute {
        return;
    }
    let start_bucket = monitoring_bucket_for_minute(start_minute);
    let end_bucket = monitoring_bucket_for_minute(end_minute.saturating_sub(1));
    for bucket in start_bucket..=end_bucket {
        *loads.entry(bucket).or_default() += load;
    }
}

fn scheduled_assignments_overlap(left: &FurnaceAssignment, right: &FurnaceAssignment) -> bool {
    let Some(left_start) = left.start_minutes() else {
        return false;
    };
    let Some(left_end) = left.end_minutes() else {
        return false;
    };
    let Some(right_start) = right.start_minutes() else {
        return false;
    };
    let Some(right_end) = right.end_minutes() else {
        return false;
    };
    left_start < right_end && right_start < left_end
}

fn scheduled_assignments_have_changeover_gap(
    left: &FurnaceAssignment,
    right: &FurnaceAssignment,
) -> bool {
    if left.furnace_idx().is_none() || left.furnace_idx() != right.furnace_idx() {
        return false;
    }
    let Some(left_end) = left.end_minutes() else {
        return false;
    };
    let Some(right_start) = right.start_minutes() else {
        return false;
    };
    if left_end > right_start {
        return false;
    }
    let required_gap = calculate_changeover_cost_for_sequence(left, right) as usize;
    right_start - left_end < required_gap
}

fn push_first_assignment_value_repair(
    plan: &Plan,
    specs: &mut Vec<RepairCandidate<Plan>>,
    assignment_index: usize,
    reason: &'static str,
) {
    if let Some(value) = first_non_conflicting_assignment_value(plan, assignment_index) {
        specs.push(RepairCandidate::new(
            reason,
            vec![Plan::assignments()
                .scalar("assignment")
                .set(assignment_index, Some(value))],
        ));
    }
}

fn first_non_conflicting_assignment_value(plan: &Plan, assignment_index: usize) -> Option<usize> {
    let assignment = &plan.assignments[assignment_index];
    assignment
        .compatible_assignments
        .iter()
        .copied()
        .find(|value| assignment_value_is_hard_clean(plan, assignment_index, *value))
}

pub(crate) fn assignment_value_is_hard_clean(
    plan: &Plan,
    assignment_index: usize,
    value: usize,
) -> bool {
    let assignment = &plan.assignments[assignment_index];
    let furnace_idx = value / NUM_TIME_SLOTS;
    let start = (value % NUM_TIME_SLOTS) * TIME_STEP;
    let end = start + assignment.duration_minutes();

    if end > HORIZON_MINUTES || !candidate_task_windows_have_owners(assignment, start, end) {
        return false;
    }

    for (other_index, other) in plan.assignments.iter().enumerate() {
        if other_index == assignment_index || other.furnace_idx() != Some(furnace_idx) {
            continue;
        }
        let Some(other_start) = other.start_minutes() else {
            continue;
        };
        let Some(other_end) = other.end_minutes() else {
            continue;
        };

        if start < other_end && other_start < end {
            return false;
        }
        if other_end <= start {
            let required_gap = calculate_changeover_cost_for_sequence(other, assignment) as usize;
            if start - other_end < required_gap {
                return false;
            }
        } else if end <= other_start {
            let required_gap = calculate_changeover_cost_for_sequence(assignment, other) as usize;
            if other_start - end < required_gap {
                return false;
            }
        }
    }

    true
}

fn candidate_task_windows_have_owners(
    assignment: &FurnaceAssignment,
    start: usize,
    end: usize,
) -> bool {
    let load_build = start
        .checked_sub(60)
        .zip(start.checked_sub(30))
        .and_then(|interval| owning_shift_identity_for_interval(interval.0, interval.1));
    let program = start
        .checked_sub(15)
        .map(|program_start| (program_start, start))
        .and_then(|interval| owning_shift_identity_for_interval(interval.0, interval.1));
    let quench = assignment
        .requires_quench
        .then_some((end, end + 15))
        .and_then(|interval| owning_shift_identity_for_interval(interval.0, interval.1));
    let unload = owning_shift_identity_for_interval(end + 15, end + 30);

    load_build.is_some()
        && program.is_some()
        && (!assignment.requires_quench || quench.is_some())
        && unload.is_some()
}
