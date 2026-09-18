use solverforge::prelude::*;

use super::constants::{RosterDay, ShiftId};
use super::enums::{OperatorRole, OperatorSkill, ShiftType};
use super::plan::Plan;
use super::support::{
    MonitoringEntry, MonitoringLoad, NightWindowEntry, ShiftCoverageEntry, ShiftWorkEntry,
    SkillMask,
};
use super::time::{
    monitoring_bucket_for_minute, shift_bounds, shift_display_start_minute,
    shift_id_for_roster_day_and_type,
};
use super::workforce::{shift_end_minute, MAX_VISIBLE_SHIFTS_PER_WEEK, MINIMUM_REST_MINUTES};

#[planning_entity]
pub struct OperatorShiftAssignment {
    #[planning_id]
    pub id: usize,
    pub operator_idx: usize,
    pub operator_name: &'static str,
    pub role: OperatorRole,
    pub day_only: bool,
    pub roster_day: RosterDay,
    pub skill_mask: SkillMask,
    pub allowed_shift_types: Vec<usize>,
    #[planning_variable(
        allows_unassigned = true,
        value_range_provider = "allowed_shift_types",
        construction_entity_order_key = "operator_shift_construction_entity_order_key",
        construction_value_order_key = "operator_shift_construction_value_order_key"
    )]
    pub shift_type: Option<usize>,
}

impl OperatorShiftAssignment {
    pub fn is_working(&self) -> bool {
        matches!(self.shift_type, Some(0..=2))
    }

    pub fn is_off(&self) -> bool {
        self.shift_type == Some(3)
    }

    pub fn shift_type_enum(&self) -> Option<ShiftType> {
        match self.shift_type {
            Some(0) => Some(ShiftType::Morning),
            Some(1) => Some(ShiftType::Afternoon),
            Some(2) => Some(ShiftType::Night),
            _ => None,
        }
    }

    pub fn shift_id(&self) -> Option<ShiftId> {
        self.shift_type_enum()
            .and_then(|shift_type| shift_id_for_roster_day_and_type(self.roster_day, shift_type))
    }

    pub fn covers(&self, skill: OperatorSkill) -> bool {
        self.skill_mask.contains(skill)
    }

    pub fn has_role(&self, role: OperatorRole) -> bool {
        self.role == role
    }
}

pub(crate) fn operator_shift_construction_entity_order_key(
    plan: &Plan,
    assignment: &OperatorShiftAssignment,
) -> i64 {
    let pressure = shift_pressure(plan, assignment);
    -pressure.saturating_mul(10_000)
        + (assignment.roster_day as i64).saturating_mul(100)
        + assignment.operator_idx as i64
}

pub(crate) fn operator_shift_construction_value_order_key(
    plan: &Plan,
    assignment: &OperatorShiftAssignment,
    shift_type_value: usize,
) -> i64 {
    if assignment.shift_type == Some(shift_type_value) {
        return i64::MIN;
    }
    if !assignment.allowed_shift_types.contains(&shift_type_value) {
        return i64::MAX;
    }

    let shift_demand_shortage = shift_type_from_value(shift_type_value)
        .and_then(|shift_type| shift_id_for_roster_day_and_type(assignment.roster_day, shift_type))
        .map(|shift_id| {
            let required = plan
                .shift_coverage_demands
                .iter()
                .filter(|demand| demand.shift_id == shift_id && demand.role == assignment.role)
                .map(|demand| demand.required_count)
                .sum::<i64>();
            let assigned = plan
                .operator_shift_assignments
                .iter()
                .filter(|other| {
                    other.id != assignment.id
                        && other.role == assignment.role
                        && other.shift_id() == Some(shift_id)
                })
                .count() as i64;
            required.saturating_sub(assigned).max(0)
        })
        .unwrap_or(0);
    let uncovered_day_role_demand = if shift_type_value == super::constants::OFF_SHIFT_VALUE {
        uncovered_day_role_demand(plan, assignment)
    } else {
        0
    };
    let demand_key = if shift_type_value == super::constants::OFF_SHIFT_VALUE {
        uncovered_day_role_demand.saturating_mul(25_000)
    } else if shift_demand_shortage > 0 {
        -shift_demand_shortage.saturating_mul(100_000)
    } else {
        25_000
    };
    let rest_penalty =
        operator_shift_rest_shortage(plan, assignment, shift_type_value).saturating_mul(20_000);
    let visible_shift_penalty =
        operator_visible_shift_excess(plan, assignment, shift_type_value).saturating_mul(900_000);

    rest_penalty + visible_shift_penalty + demand_key + shift_type_value as i64
}

fn uncovered_day_role_demand(plan: &Plan, assignment: &OperatorShiftAssignment) -> i64 {
    plan.shift_coverage_demands
        .iter()
        .filter(|demand| {
            demand.role == assignment.role
                && plan
                    .shift_by_id(demand.shift_id)
                    .is_some_and(|shift| shift.roster_day == assignment.roster_day)
        })
        .map(|demand| {
            let assigned = plan
                .operator_shift_assignments
                .iter()
                .filter(|other| {
                    other.id != assignment.id
                        && other.role == assignment.role
                        && other.shift_id() == Some(demand.shift_id)
                })
                .count() as i64;
            demand.required_count.saturating_sub(assigned).max(0)
        })
        .sum()
}

fn shift_pressure(plan: &Plan, assignment: &OperatorShiftAssignment) -> i64 {
    plan.shift_coverage_demands
        .iter()
        .filter(|demand| {
            plan.shift_by_id(demand.shift_id)
                .is_some_and(|shift| shift.roster_day == assignment.roster_day)
                && demand.role == assignment.role
        })
        .map(|demand| demand.required_count)
        .sum()
}

fn shift_type_from_value(value: usize) -> Option<ShiftType> {
    match value {
        0 => Some(ShiftType::Morning),
        1 => Some(ShiftType::Afternoon),
        2 => Some(ShiftType::Night),
        _ => None,
    }
}

fn operator_shift_rest_shortage(
    plan: &Plan,
    assignment: &OperatorShiftAssignment,
    shift_type_value: usize,
) -> i64 {
    let Some(candidate_shift_type) = shift_type_from_value(shift_type_value) else {
        return 0;
    };
    let Some(candidate_end) = shift_end_minute(assignment.roster_day, candidate_shift_type) else {
        return 0;
    };
    let candidate_start = shift_display_start_minute(assignment.roster_day, candidate_shift_type);
    let mut shortage = 0i64;

    for neighbor in plan.operator_shift_assignments.iter().filter(|neighbor| {
        neighbor.id != assignment.id && neighbor.operator_idx == assignment.operator_idx
    }) {
        let Some(neighbor_shift_type) = neighbor.shift_type_enum() else {
            continue;
        };
        if neighbor.roster_day + 1 == assignment.roster_day {
            if let Some(previous_end) = shift_end_minute(neighbor.roster_day, neighbor_shift_type) {
                let rest = candidate_start - previous_end;
                shortage += (MINIMUM_REST_MINUTES - rest).max(0) as i64;
            }
        } else if assignment.roster_day + 1 == neighbor.roster_day {
            let next_start = shift_display_start_minute(neighbor.roster_day, neighbor_shift_type);
            let rest = next_start - candidate_end;
            shortage += (MINIMUM_REST_MINUTES - rest).max(0) as i64;
        }
    }

    shortage
}

fn operator_visible_shift_excess(
    plan: &Plan,
    assignment: &OperatorShiftAssignment,
    shift_type_value: usize,
) -> i64 {
    if assignment.roster_day < 0 || shift_type_from_value(shift_type_value).is_none() {
        return 0;
    }

    let visible_working_shifts = plan
        .operator_shift_assignments
        .iter()
        .filter(|other| {
            other.id != assignment.id
                && other.operator_idx == assignment.operator_idx
                && other.roster_day >= 0
                && other.is_working()
        })
        .count()
        + 1;

    visible_working_shifts.saturating_sub(MAX_VISIBLE_SHIFTS_PER_WEEK) as i64
}

pub struct OperatorMonitoringCapacityEntries;

impl Projection<OperatorShiftAssignment> for OperatorMonitoringCapacityEntries {
    type Out = MonitoringEntry;
    const MAX_EMITS: usize = 672;

    fn project<Sink>(&self, assignment: &OperatorShiftAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        if !assignment.is_working() || !assignment.covers(OperatorSkill::Monitor) {
            return;
        }

        let Some(shift_type) = assignment.shift_type_enum() else {
            return;
        };
        let Some((start_minute, end_minute)) = shift_bounds(assignment.roster_day, shift_type)
        else {
            return;
        };

        emit_monitoring_entries(
            out,
            start_minute,
            end_minute,
            MonitoringLoad {
                capacity: 1,
                ..MonitoringLoad::default()
            },
        );
    }
}

pub struct OperatorTaskShiftWorkEntries;

impl Projection<OperatorShiftAssignment> for OperatorTaskShiftWorkEntries {
    type Out = ShiftWorkEntry;
    const MAX_EMITS: usize = 1;

    fn project<Sink>(&self, assignment: &OperatorShiftAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        let Some(shift_id) = assignment.shift_id() else {
            return;
        };
        out.emit(ShiftWorkEntry {
            operator_id: assignment.operator_idx,
            shift_id,
            delta: -1_000,
        });
    }
}

pub struct OperatorShiftCoverageEntries;

impl Projection<OperatorShiftAssignment> for OperatorShiftCoverageEntries {
    type Out = ShiftCoverageEntry;
    const MAX_EMITS: usize = 1;

    fn project<Sink>(&self, assignment: &OperatorShiftAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        let Some(shift_id) = assignment.shift_id() else {
            return;
        };
        out.emit(ShiftCoverageEntry {
            shift_id,
            role: assignment.role,
            delta: -1,
        });
    }
}

pub struct OperatorNightWindowEntries;

impl Projection<OperatorShiftAssignment> for OperatorNightWindowEntries {
    type Out = NightWindowEntry;
    const MAX_EMITS: usize = 4;

    fn project<Sink>(&self, assignment: &OperatorShiftAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        if assignment.roster_day < 0 || assignment.shift_type_enum() != Some(ShiftType::Night) {
            return;
        }

        let first_window_start = assignment.roster_day.saturating_sub(3).max(0);
        for window_start_day in first_window_start..=assignment.roster_day {
            out.emit(NightWindowEntry {
                operator_id: assignment.operator_idx,
                window_start_day,
                delta: 1,
            });
        }
    }
}

fn emit_monitoring_entries(
    out: &mut impl ProjectionSink<MonitoringEntry>,
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
        out.emit(MonitoringEntry { bucket, load });
    }
}

impl OperatorShiftAssignment {
    pub fn finalize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::build_demo_problem;
    use crate::domain::OFF_SHIFT_VALUE;

    #[test]
    fn roster_construction_value_key_avoids_minimum_rest_violations() {
        let mut plan = build_demo_problem();
        let previous_index = plan
            .operator_shift_assignments
            .iter()
            .position(|assignment| assignment.operator_idx == 0 && assignment.roster_day == 1)
            .expect("previous roster day");
        let current_index = plan
            .operator_shift_assignments
            .iter()
            .position(|assignment| assignment.operator_idx == 0 && assignment.roster_day == 2)
            .expect("current roster day");
        plan.operator_shift_assignments[previous_index].shift_type =
            Some(ShiftType::Afternoon.index());
        plan.operator_shift_assignments[current_index].shift_type = None;

        let current = &plan.operator_shift_assignments[current_index];
        let morning_key =
            operator_shift_construction_value_order_key(&plan, current, ShiftType::Morning.index());
        let afternoon_key = operator_shift_construction_value_order_key(
            &plan,
            current,
            ShiftType::Afternoon.index(),
        );

        assert!(
            morning_key > afternoon_key,
            "morning after afternoon should be less attractive than a rested shift"
        );
    }

    #[test]
    fn roster_construction_value_key_avoids_seventh_visible_shift() {
        let mut plan = build_demo_problem();
        for day in 0..=5 {
            let index = plan
                .operator_shift_assignments
                .iter()
                .position(|assignment| assignment.operator_idx == 0 && assignment.roster_day == day)
                .expect("visible roster day");
            plan.operator_shift_assignments[index].shift_type = Some(ShiftType::Morning.index());
        }
        let seventh_index = plan
            .operator_shift_assignments
            .iter()
            .position(|assignment| assignment.operator_idx == 0 && assignment.roster_day == 6)
            .expect("seventh visible roster day");
        plan.operator_shift_assignments[seventh_index].shift_type = None;

        let seventh = &plan.operator_shift_assignments[seventh_index];
        let working_key = operator_shift_construction_value_order_key(
            &plan,
            seventh,
            ShiftType::Afternoon.index(),
        );
        let off_key = operator_shift_construction_value_order_key(&plan, seventh, OFF_SHIFT_VALUE);

        assert!(
            working_key > off_key,
            "a seventh visible shift should be less attractive than off"
        );
    }
}
