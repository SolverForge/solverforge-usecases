use solverforge::prelude::*;

use super::changeover::calculate_changeover_cost_for_sequence;
use super::constants::{NUM_TIME_SLOTS, TIME_STEP};
use super::enums::{HeatTreatmentProcess, OperatorSkill, PriorityBand, ShiftType};
use super::plan::Plan;
use super::support::{
    HardViolation, ManualTaskBooking, ManualTaskDemand, ManualTaskKind, MonitoringEntry,
    MonitoringLoad, ShiftIdentity, ShiftWorkEntry, SkillMask,
};
use super::time::{
    heating_ramp_minutes, monitoring_bucket_for_minute, owning_shift_identity_for_interval,
    owning_shift_identity_for_minute, shift_identities_for_interval,
};

#[planning_entity]
pub struct FurnaceAssignment {
    #[planning_id]
    pub work_order_idx: usize,
    pub process: HeatTreatmentProcess,
    pub temperature_celsius: u32,
    pub soak_time_minutes: u32,
    pub load_weight_kg: u32,
    pub due_datetime_minutes: usize,
    pub priority: PriorityBand,
    pub requires_quench: bool,
    pub compatible_assignments: Vec<usize>,
    #[planning_variable(
        allows_unassigned = true,
        value_range_provider = "compatible_assignments",
        construction_entity_order_key = "furnace_assignment_construction_entity_order_key",
        construction_value_order_key = "furnace_assignment_construction_value_order_key"
    )]
    pub assignment: Option<usize>,
    pub eligible_load_build_operator_ids: Vec<usize>,
    pub eligible_program_operator_ids: Vec<usize>,
    pub eligible_quench_operator_ids: Vec<usize>,
    pub eligible_unload_operator_ids: Vec<usize>,
    #[planning_variable(
        allows_unassigned = true,
        value_range_provider = "eligible_load_build_operator_ids",
        construction_entity_order_key = "task_operator_construction_entity_order_key",
        construction_value_order_key = "load_build_operator_construction_value_order_key"
    )]
    pub load_build_operator_id: Option<usize>,
    #[planning_variable(
        allows_unassigned = true,
        value_range_provider = "eligible_program_operator_ids",
        construction_entity_order_key = "task_operator_construction_entity_order_key",
        construction_value_order_key = "program_operator_construction_value_order_key"
    )]
    pub program_operator_id: Option<usize>,
    #[planning_variable(
        allows_unassigned = true,
        value_range_provider = "eligible_quench_operator_ids",
        construction_entity_order_key = "task_operator_construction_entity_order_key",
        construction_value_order_key = "quench_operator_construction_value_order_key"
    )]
    pub quench_operator_id: Option<usize>,
    #[planning_variable(
        allows_unassigned = true,
        value_range_provider = "eligible_unload_operator_ids",
        construction_entity_order_key = "task_operator_construction_entity_order_key",
        construction_value_order_key = "unload_operator_construction_value_order_key"
    )]
    pub unload_operator_id: Option<usize>,
}

impl FurnaceAssignment {
    pub fn is_scheduled(&self) -> bool {
        self.assignment.is_some()
    }

    pub fn furnace_idx(&self) -> Option<usize> {
        self.assignment.map(|value| value / NUM_TIME_SLOTS)
    }

    pub fn time_slot_idx(&self) -> Option<usize> {
        self.assignment.map(|value| value % NUM_TIME_SLOTS)
    }

    pub fn start_minutes(&self) -> Option<usize> {
        self.time_slot_idx().map(|slot| slot * TIME_STEP)
    }

    pub fn end_minutes(&self) -> Option<usize> {
        self.start_minutes()
            .map(|start| start + self.soak_time_minutes as usize)
    }

    pub fn duration_minutes(&self) -> usize {
        self.soak_time_minutes as usize
    }

    pub fn priority_weight(&self) -> i64 {
        self.priority.weight()
    }

    pub fn start_shift_identity(&self) -> Option<ShiftIdentity> {
        self.start_minutes()
            .and_then(owning_shift_identity_for_minute)
    }

    pub fn completion_shift_identity(&self) -> Option<ShiftIdentity> {
        self.end_minutes()
            .and_then(owning_shift_identity_for_minute)
    }

    pub fn load_build_interval(&self) -> Option<(usize, usize)> {
        let start = self.start_minutes()?;
        let interval_start = start.saturating_sub(60);
        let interval_end = start.saturating_sub(30);
        (interval_end > interval_start).then_some((interval_start, interval_end))
    }

    pub fn program_interval(&self) -> Option<(usize, usize)> {
        let start = self.start_minutes()?;
        let interval_start = start.saturating_sub(15);
        (start > interval_start).then_some((interval_start, start))
    }

    pub fn quench_interval(&self) -> Option<(usize, usize)> {
        if !self.requires_quench {
            return None;
        }
        let end = self.end_minutes()?;
        Some((end, end + 15))
    }

    pub fn unload_interval(&self) -> Option<(usize, usize)> {
        let end = self.end_minutes()?;
        Some((end + 15, end + 30))
    }

    pub fn load_build_shift_identity(&self) -> Option<ShiftIdentity> {
        self.load_build_interval()
            .and_then(|(start, end)| owning_shift_identity_for_interval(start, end))
    }

    pub fn program_shift_identity(&self) -> Option<ShiftIdentity> {
        self.program_interval()
            .and_then(|(start, end)| owning_shift_identity_for_interval(start, end))
    }

    pub fn quench_shift_identity(&self) -> Option<ShiftIdentity> {
        self.quench_interval()
            .and_then(|(start, end)| owning_shift_identity_for_interval(start, end))
    }

    pub fn unload_shift_identity(&self) -> Option<ShiftIdentity> {
        self.unload_interval()
            .and_then(|(start, end)| owning_shift_identity_for_interval(start, end))
    }

    pub fn occupied_shift_identities(&self) -> Vec<ShiftIdentity> {
        let Some(start) = self.start_minutes() else {
            return Vec::new();
        };
        let Some(end) = self.end_minutes() else {
            return Vec::new();
        };
        shift_identities_for_interval(start, end)
    }

    pub fn task_interval(&self, kind: ManualTaskKind) -> Option<(usize, usize)> {
        match kind {
            ManualTaskKind::LoadBuild => self.load_build_interval(),
            ManualTaskKind::Program => self.program_interval(),
            ManualTaskKind::Quench => self.quench_interval(),
            ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => self.unload_interval(),
        }
    }

    pub fn task_shift_identity(&self, kind: ManualTaskKind) -> Option<ShiftIdentity> {
        self.task_interval(kind)
            .and_then(|(start, end)| owning_shift_identity_for_interval(start, end))
    }

    pub fn load_build_required_skills(&self) -> SkillMask {
        SkillMask::empty().insert(OperatorSkill::LoadBuild)
    }

    pub fn program_required_skills(&self) -> SkillMask {
        SkillMask::empty().insert(OperatorSkill::FurnaceProgram)
    }

    pub fn quench_required_skills(&self) -> SkillMask {
        SkillMask::empty().insert(OperatorSkill::QuenchOperate)
    }

    pub fn unload_required_skills(&self) -> SkillMask {
        if self.load_weight_kg > 500 {
            SkillMask::empty()
                .insert(OperatorSkill::LoadBuild)
                .insert(OperatorSkill::CraneForklift)
        } else {
            SkillMask::empty().insert(OperatorSkill::LoadBuild)
        }
    }

    pub fn task_required_skills(&self, kind: ManualTaskKind) -> SkillMask {
        match kind {
            ManualTaskKind::LoadBuild => self.load_build_required_skills(),
            ManualTaskKind::Program => self.program_required_skills(),
            ManualTaskKind::Quench => self.quench_required_skills(),
            ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => self.unload_required_skills(),
        }
    }

    pub fn unload_task_kind(&self) -> ManualTaskKind {
        if self.load_weight_kg > 500 {
            ManualTaskKind::HeavyUnload
        } else {
            ManualTaskKind::Unload
        }
    }

    pub fn task_operator_id(&self, kind: ManualTaskKind) -> Option<usize> {
        match kind {
            ManualTaskKind::LoadBuild => self.load_build_operator_id,
            ManualTaskKind::Program => self.program_operator_id,
            ManualTaskKind::Quench => self.quench_operator_id,
            ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => self.unload_operator_id,
        }
    }

    pub fn task_eligible_operator_ids(&self, kind: ManualTaskKind) -> &[usize] {
        match kind {
            ManualTaskKind::LoadBuild => &self.eligible_load_build_operator_ids,
            ManualTaskKind::Program => &self.eligible_program_operator_ids,
            ManualTaskKind::Quench => &self.eligible_quench_operator_ids,
            ManualTaskKind::Unload | ManualTaskKind::HeavyUnload => {
                &self.eligible_unload_operator_ids
            }
        }
    }

    pub fn required_task_kinds(&self) -> Vec<ManualTaskKind> {
        if !self.is_scheduled() {
            return Vec::new();
        }

        let mut kinds = Vec::with_capacity(4);
        if self.load_build_interval().is_some() {
            kinds.push(ManualTaskKind::LoadBuild);
        }
        if self.program_interval().is_some() {
            kinds.push(ManualTaskKind::Program);
        }
        if self.quench_interval().is_some() {
            kinds.push(ManualTaskKind::Quench);
        }
        if self.unload_interval().is_some() {
            kinds.push(self.unload_task_kind());
        }
        kinds
    }

    pub fn manual_task_demands(&self) -> Vec<ManualTaskDemand> {
        let mut demands = Vec::new();
        self.for_each_manual_task_demand(|demand| demands.push(demand));
        demands
    }

    pub fn for_each_manual_task_demand(&self, mut emit: impl FnMut(ManualTaskDemand)) {
        let Some(furnace_idx) = self.furnace_idx() else {
            return;
        };

        push_manual_task_with(
            self.work_order_idx,
            furnace_idx,
            self.load_build_interval(),
            ManualTaskKind::LoadBuild,
            self.load_build_required_skills(),
            &mut emit,
        );
        push_manual_task_with(
            self.work_order_idx,
            furnace_idx,
            self.program_interval(),
            ManualTaskKind::Program,
            self.program_required_skills(),
            &mut emit,
        );
        push_manual_task_with(
            self.work_order_idx,
            furnace_idx,
            self.quench_interval(),
            ManualTaskKind::Quench,
            self.quench_required_skills(),
            &mut emit,
        );
        push_manual_task_with(
            self.work_order_idx,
            furnace_idx,
            self.unload_interval(),
            self.unload_task_kind(),
            self.unload_required_skills(),
            &mut emit,
        );
    }

    pub fn manual_task_bookings(&self) -> Vec<ManualTaskBooking> {
        let mut bookings = Vec::new();
        self.for_each_manual_task_booking(|booking| bookings.push(booking));
        bookings
    }

    pub fn for_each_manual_task_booking(&self, mut emit: impl FnMut(ManualTaskBooking)) {
        let Some(furnace_idx) = self.furnace_idx() else {
            return;
        };

        push_manual_task_booking_with(
            self,
            furnace_idx,
            self.load_build_interval(),
            ManualTaskKind::LoadBuild,
            self.load_build_required_skills(),
            &mut emit,
        );
        push_manual_task_booking_with(
            self,
            furnace_idx,
            self.program_interval(),
            ManualTaskKind::Program,
            self.program_required_skills(),
            &mut emit,
        );
        push_manual_task_booking_with(
            self,
            furnace_idx,
            self.quench_interval(),
            ManualTaskKind::Quench,
            self.quench_required_skills(),
            &mut emit,
        );
        push_manual_task_booking_with(
            self,
            furnace_idx,
            self.unload_interval(),
            self.unload_task_kind(),
            self.unload_required_skills(),
            &mut emit,
        );
    }

    pub fn task_operator_range_violations(&self) -> Vec<HardViolation> {
        [
            (ManualTaskKind::LoadBuild, self.load_build_operator_id),
            (ManualTaskKind::Program, self.program_operator_id),
            (ManualTaskKind::Quench, self.quench_operator_id),
            (self.unload_task_kind(), self.unload_operator_id),
        ]
        .into_iter()
        .filter_map(|(kind, operator_id)| {
            let operator_id = operator_id?;
            (!self.task_eligible_operator_ids(kind).contains(&operator_id))
                .then_some(HardViolation { weight: 900 })
        })
        .collect()
    }

    pub fn finalize(&mut self) {}
}

pub(crate) fn furnace_assignment_construction_entity_order_key(
    _plan: &Plan,
    assignment: &FurnaceAssignment,
) -> i64 {
    let range_size = assignment.compatible_assignments.len() as i64;
    let due = assignment.due_datetime_minutes as i64;
    let duration = assignment.duration_minutes() as i64;
    -due.saturating_mul(10_000) - range_size.saturating_mul(100) + duration
}

pub(crate) fn furnace_assignment_construction_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    value: usize,
) -> i64 {
    if assignment.assignment == Some(value) {
        return i64::MIN;
    }

    let furnace_idx = value / NUM_TIME_SLOTS;
    let slot_idx = value % NUM_TIME_SLOTS;
    let start = slot_idx * TIME_STEP;
    let end = start + assignment.duration_minutes();
    let due_distance = assignment.due_datetime_minutes.abs_diff(end) as i64;
    let schedule_conflict =
        furnace_candidate_schedule_conflict(plan, assignment, furnace_idx, start, end);
    let shift_gap = usize::from(candidate_has_shift_ownership_gap(assignment, start, end)) as i64;
    let rotated_furnace_rank = if plan.furnaces.is_empty() {
        0
    } else {
        ((furnace_idx + assignment.work_order_idx) % plan.furnaces.len()) as i64
    };

    schedule_conflict.saturating_mul(100_000_000)
        + shift_gap.saturating_mul(10_000_000)
        + due_distance.saturating_mul(1_000)
        + start as i64
        + rotated_furnace_rank
}

pub(crate) fn furnace_assignment_local_search_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    value: usize,
) -> i64 {
    if assignment.assignment == Some(value) {
        return i64::MIN;
    }

    let furnace_idx = value / NUM_TIME_SLOTS;
    let slot_idx = value % NUM_TIME_SLOTS;
    let start = slot_idx * TIME_STEP;
    let end = start + assignment.duration_minutes();
    let timing_penalty = furnace_candidate_timing_soft_proxy(assignment, start, end);
    let schedule_conflict =
        furnace_candidate_schedule_conflict(plan, assignment, furnace_idx, start, end);
    let shift_gap = usize::from(candidate_has_shift_ownership_gap(assignment, start, end)) as i64;
    let rotated_furnace_rank = if plan.furnaces.is_empty() {
        0
    } else {
        ((furnace_idx + assignment.work_order_idx) % plan.furnaces.len()) as i64
    };

    schedule_conflict.saturating_mul(100_000_000)
        + shift_gap.saturating_mul(10_000_000)
        + timing_penalty.saturating_mul(10_000)
        + start as i64
        + rotated_furnace_rank
}

fn furnace_candidate_timing_soft_proxy(
    assignment: &FurnaceAssignment,
    start: usize,
    end: usize,
) -> i64 {
    let due_penalty = if end > assignment.due_datetime_minutes {
        (end - assignment.due_datetime_minutes) as i64 * priority_lateness_multiplier(assignment)
    } else {
        ((assignment.due_datetime_minutes - end) as i64) / 40
    };
    due_penalty + night_start_penalty(assignment, start)
}

fn priority_lateness_multiplier(assignment: &FurnaceAssignment) -> i64 {
    match assignment.priority {
        PriorityBand::Express => 60,
        PriorityBand::Urgent => 20,
        PriorityBand::Standard => 5,
    }
}

fn night_start_penalty(assignment: &FurnaceAssignment, start: usize) -> i64 {
    let starts_at_night = owning_shift_identity_for_minute(start)
        .is_some_and(|shift| shift.shift_type == ShiftType::Night);
    if !starts_at_night
        || matches!(
            assignment.process,
            HeatTreatmentProcess::Carburizing | HeatTreatmentProcess::Nitriding
        )
    {
        return 0;
    }
    match assignment.priority {
        PriorityBand::Express => 40,
        PriorityBand::Urgent => 70,
        PriorityBand::Standard => 120,
    }
}

fn furnace_candidate_schedule_conflict(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    furnace_idx: usize,
    start: usize,
    end: usize,
) -> i64 {
    let mut penalty = 0_i64;
    for other in &plan.assignments {
        if other.work_order_idx == assignment.work_order_idx
            || other.furnace_idx() != Some(furnace_idx)
        {
            continue;
        }
        let Some(other_start) = other.start_minutes() else {
            continue;
        };
        let Some(other_end) = other.end_minutes() else {
            continue;
        };

        if start < other_end && other_start < end {
            penalty = penalty
                .saturating_add(10_000 + overlap_minutes(start, end, other_start, other_end));
            continue;
        }
        if other_end <= start {
            let required_gap = calculate_changeover_cost_for_sequence(other, assignment) as usize;
            if start - other_end < required_gap {
                penalty = penalty.saturating_add((required_gap - (start - other_end)) as i64);
            }
        } else if end <= other_start {
            let required_gap = calculate_changeover_cost_for_sequence(assignment, other) as usize;
            if other_start - end < required_gap {
                penalty = penalty.saturating_add((required_gap - (other_start - end)) as i64);
            }
        }
    }
    penalty
}

fn overlap_minutes(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> i64 {
    left_end
        .min(right_end)
        .saturating_sub(left_start.max(right_start)) as i64
}

pub(crate) fn task_operator_construction_entity_order_key(
    _plan: &Plan,
    assignment: &FurnaceAssignment,
) -> i64 {
    let start = assignment.start_minutes().unwrap_or(usize::MAX) as i64;
    let range_size = [
        assignment.eligible_load_build_operator_ids.len(),
        assignment.eligible_program_operator_ids.len(),
        assignment.eligible_quench_operator_ids.len(),
        assignment.eligible_unload_operator_ids.len(),
    ]
    .into_iter()
    .filter(|size| *size > 0)
    .min()
    .unwrap_or(usize::MAX) as i64;

    start.saturating_mul(10_000) + range_size.saturating_mul(100) + assignment.work_order_idx as i64
}

pub(crate) fn load_build_operator_construction_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    operator_id: usize,
) -> i64 {
    task_operator_construction_value_order_key(
        plan,
        assignment,
        ManualTaskKind::LoadBuild,
        operator_id,
    )
}

pub(crate) fn program_operator_construction_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    operator_id: usize,
) -> i64 {
    task_operator_construction_value_order_key(
        plan,
        assignment,
        ManualTaskKind::Program,
        operator_id,
    )
}

pub(crate) fn quench_operator_construction_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    operator_id: usize,
) -> i64 {
    task_operator_construction_value_order_key(
        plan,
        assignment,
        ManualTaskKind::Quench,
        operator_id,
    )
}

pub(crate) fn unload_operator_construction_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    operator_id: usize,
) -> i64 {
    task_operator_construction_value_order_key(
        plan,
        assignment,
        assignment.unload_task_kind(),
        operator_id,
    )
}

fn task_operator_construction_value_order_key(
    plan: &Plan,
    assignment: &FurnaceAssignment,
    kind: ManualTaskKind,
    operator_id: usize,
) -> i64 {
    let shift_fit = assignment
        .task_shift_identity(kind)
        .map(|shift| operator_shift_fit(plan, operator_id, shift))
        .unwrap_or(9);
    let shift_conflict = assignment
        .task_shift_identity(kind)
        .map(|shift| operator_existing_shift_conflict(plan, operator_id, shift))
        .unwrap_or(0);
    let overlap_conflict = assignment
        .task_interval(kind)
        .map(|(start, end)| operator_existing_task_overlap(plan, operator_id, start, end))
        .unwrap_or(0);
    let current_task_count = plan
        .assignments
        .iter()
        .filter(|other| {
            [
                other.load_build_operator_id,
                other.program_operator_id,
                other.quench_operator_id,
                other.unload_operator_id,
            ]
            .contains(&Some(operator_id))
        })
        .count() as i64;

    shift_conflict.saturating_mul(100_000_000)
        + overlap_conflict.saturating_mul(10_000_000)
        + shift_fit.saturating_mul(1_000_000)
        + current_task_count.saturating_mul(1_000)
        + operator_id as i64
}

fn operator_shift_fit(plan: &Plan, operator_id: usize, shift: ShiftIdentity) -> i64 {
    let Some(shift_assignment) = plan.operator_shift_assignments.iter().find(|assignment| {
        assignment.operator_idx == operator_id && assignment.roster_day == shift.roster_day
    }) else {
        return 9;
    };

    if shift_assignment.shift_id() == Some(shift.shift_id) {
        0
    } else if shift_assignment
        .allowed_shift_types
        .contains(&shift.shift_type.index())
    {
        1
    } else {
        9
    }
}

fn operator_existing_shift_conflict(plan: &Plan, operator_id: usize, shift: ShiftIdentity) -> i64 {
    for assignment in &plan.assignments {
        for kind in assignment.required_task_kinds() {
            if assignment.task_operator_id(kind) != Some(operator_id) {
                continue;
            }
            let Some(existing_shift) = assignment.task_shift_identity(kind) else {
                continue;
            };
            if existing_shift.roster_day == shift.roster_day
                && existing_shift.shift_id != shift.shift_id
            {
                return 1;
            }
        }
    }
    0
}

fn operator_existing_task_overlap(
    plan: &Plan,
    operator_id: usize,
    start: usize,
    end: usize,
) -> i64 {
    for assignment in &plan.assignments {
        for kind in assignment.required_task_kinds() {
            if assignment.task_operator_id(kind) != Some(operator_id) {
                continue;
            }
            let Some((existing_start, existing_end)) = assignment.task_interval(kind) else {
                continue;
            };
            if start < existing_end && existing_start < end {
                return 1;
            }
        }
    }
    0
}

fn candidate_has_shift_ownership_gap(
    assignment: &FurnaceAssignment,
    start: usize,
    end: usize,
) -> bool {
    let load_build = (start >= 60)
        .then(|| (start - 60, start - 30))
        .and_then(|interval| owning_shift_identity_for_interval(interval.0, interval.1));
    let program = (start >= 15)
        .then(|| (start - 15, start))
        .and_then(|interval| owning_shift_identity_for_interval(interval.0, interval.1));
    let quench = assignment
        .requires_quench
        .then_some((end, end + 15))
        .and_then(|interval| owning_shift_identity_for_interval(interval.0, interval.1));
    let unload = owning_shift_identity_for_interval(end + 15, end + 30);

    load_build.is_none()
        || program.is_none()
        || (assignment.requires_quench && quench.is_none())
        || unload.is_none()
}

pub struct AssignedTaskCapacityEntries;

impl Projection<FurnaceAssignment> for AssignedTaskCapacityEntries {
    type Out = MonitoringEntry;
    const MAX_EMITS: usize = 2_688;

    fn project<Sink>(&self, assignment: &FurnaceAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        assignment.for_each_manual_task_booking(|booking| {
            emit_monitoring_entries(
                out,
                booking.start_minute,
                booking.end_minute,
                MonitoringLoad {
                    task: 1,
                    ..MonitoringLoad::default()
                },
            );
        });
    }
}

pub struct ManualTaskBookingEntries;

impl Projection<FurnaceAssignment> for ManualTaskBookingEntries {
    type Out = ManualTaskBooking;
    const MAX_EMITS: usize = 4;

    fn project<Sink>(&self, assignment: &FurnaceAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        assignment.for_each_manual_task_booking(|booking| out.emit(booking));
    }
}

pub struct TaskShiftWorkEntries;

impl Projection<FurnaceAssignment> for TaskShiftWorkEntries {
    type Out = ShiftWorkEntry;
    const MAX_EMITS: usize = 4;

    fn project<Sink>(&self, assignment: &FurnaceAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        assignment.for_each_manual_task_booking(|booking| {
            out.emit(ShiftWorkEntry {
                operator_id: booking.operator_id,
                shift_id: booking.shift.shift_id,
                delta: 1,
            });
        });
    }
}

pub struct AssignedTaskWithoutScheduleViolations;

impl Projection<FurnaceAssignment> for AssignedTaskWithoutScheduleViolations {
    type Out = HardViolation;
    const MAX_EMITS: usize = 4;

    fn project<Sink>(&self, assignment: &FurnaceAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        if assignment.assignment.is_some() {
            return;
        }
        let assigned_count = [
            assignment.load_build_operator_id,
            assignment.program_operator_id,
            assignment.quench_operator_id,
            assignment.unload_operator_id,
        ]
        .into_iter()
        .filter(Option::is_some)
        .count();
        for _ in 0..assigned_count {
            out.emit(HardViolation { weight: 900 });
        }
    }
}

pub struct MissingTaskOperatorViolations;

impl Projection<FurnaceAssignment> for MissingTaskOperatorViolations {
    type Out = HardViolation;
    const MAX_EMITS: usize = 4;

    fn project<Sink>(&self, assignment: &FurnaceAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        for kind in assignment.required_task_kinds() {
            if assignment.task_operator_id(kind).is_none() {
                out.emit(HardViolation { weight: 4_500 });
            }
        }
    }
}

pub struct MonitoringDemandEntries;

impl Projection<FurnaceAssignment> for MonitoringDemandEntries {
    type Out = MonitoringEntry;
    const MAX_EMITS: usize = 1_344;

    fn project<Sink>(&self, assignment: &FurnaceAssignment, out: &mut Sink)
    where
        Sink: ProjectionSink<Self::Out>,
    {
        let Some(start) = assignment.start_minutes() else {
            return;
        };
        let Some(end) = assignment.end_minutes() else {
            return;
        };

        let ramp_end = start + heating_ramp_minutes(assignment.duration_minutes());
        emit_monitoring_entries(
            out,
            start,
            ramp_end,
            MonitoringLoad {
                ramp: 1,
                ..MonitoringLoad::default()
            },
        );
        emit_monitoring_entries(
            out,
            ramp_end,
            end,
            MonitoringLoad {
                soak: 1,
                ..MonitoringLoad::default()
            },
        );
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

fn push_manual_task_with(
    work_order_idx: usize,
    furnace_idx: usize,
    interval: Option<(usize, usize)>,
    kind: ManualTaskKind,
    required_skills: SkillMask,
    emit: &mut impl FnMut(ManualTaskDemand),
) {
    let Some((start_minute, end_minute)) = interval else {
        return;
    };
    let Some(shift) = owning_shift_identity_for_interval(start_minute, end_minute) else {
        return;
    };
    emit(ManualTaskDemand {
        work_order_idx,
        furnace_idx,
        shift,
        kind,
        start_minute,
        end_minute,
        required_skills,
    });
}

fn push_manual_task_booking_with(
    assignment: &FurnaceAssignment,
    furnace_idx: usize,
    interval: Option<(usize, usize)>,
    kind: ManualTaskKind,
    required_skills: SkillMask,
    emit: &mut impl FnMut(ManualTaskBooking),
) {
    let Some(operator_id) = assignment.task_operator_id(kind) else {
        return;
    };
    let Some((start_minute, end_minute)) = interval else {
        return;
    };
    let Some(shift) = owning_shift_identity_for_interval(start_minute, end_minute) else {
        return;
    };
    emit(ManualTaskBooking {
        work_order_idx: assignment.work_order_idx,
        furnace_idx,
        operator_id,
        shift,
        kind,
        start_minute,
        end_minute,
        required_skills,
    });
}
