use super::constants::{
    RosterDay, CARRY_OUT_END_MINUTE, MINUTES_PER_DAY, OFF_SHIFT_VALUE, STAFFING_ROSTER_DAY_END,
    STAFFING_ROSTER_DAY_START,
};
use super::coverage::OperatorShiftAssignment;
use super::enums::{HeatTreatmentProcess, OperatorRole, OperatorSkill, ShiftType};
use super::support::SkillMask;
use super::time::{shift_display_start_minute, shift_id_for_roster_day_and_type};

pub const MINIMUM_REST_MINUTES: i32 = 660;
pub const MAX_VISIBLE_SHIFTS_PER_WEEK: usize = 6;
pub const TARGET_VISIBLE_SHIFTS_PER_WEEK: usize = 5;
pub const MAX_CONSECUTIVE_NIGHTS: usize = 3;

pub fn requires_quench(process: HeatTreatmentProcess) -> bool {
    matches!(
        process,
        HeatTreatmentProcess::Quenching
            | HeatTreatmentProcess::Carburizing
            | HeatTreatmentProcess::SolutionTreating
    )
}

pub fn base_role_skill_mask(role: OperatorRole) -> SkillMask {
    match role {
        OperatorRole::ShiftLead => SkillMask::empty()
            .insert(OperatorSkill::FurnaceProgram)
            .insert(OperatorSkill::LoadBuild)
            .insert(OperatorSkill::QuenchOperate)
            .insert(OperatorSkill::Monitor),
        OperatorRole::FurnaceOperator => SkillMask::empty()
            .insert(OperatorSkill::LoadBuild)
            .insert(OperatorSkill::CraneForklift)
            .insert(OperatorSkill::QuenchOperate)
            .insert(OperatorSkill::Monitor),
        OperatorRole::MaintenanceTechnician => SkillMask::empty()
            .insert(OperatorSkill::Electrical)
            .insert(OperatorSkill::Mechanical)
            .insert(OperatorSkill::Monitor),
        OperatorRole::MaterialHandler => SkillMask::empty()
            .insert(OperatorSkill::CraneForklift)
            .insert(OperatorSkill::ReceiveStage)
            .insert(OperatorSkill::ShipStore),
        OperatorRole::QualityControl => SkillMask::empty().insert(OperatorSkill::QcInspect),
    }
}

pub fn allowed_shift_types_for(
    role: OperatorRole,
    day_only: bool,
    roster_day: RosterDay,
) -> Vec<usize> {
    let mut values = vec![OFF_SHIFT_VALUE];

    if roster_day == STAFFING_ROSTER_DAY_START {
        if !day_only {
            values.push(ShiftType::Night.index());
        }
        return values;
    }

    values.push(ShiftType::Morning.index());
    values.push(ShiftType::Afternoon.index());

    if !day_only
        && !matches!(
            role,
            OperatorRole::MaterialHandler | OperatorRole::QualityControl
        )
    {
        values.push(ShiftType::Night.index());
    }

    values
}

pub fn shift_rest_minutes(
    previous: &OperatorShiftAssignment,
    current: &OperatorShiftAssignment,
) -> Option<i32> {
    let previous_type = previous.shift_type_enum()?;
    let current_type = current.shift_type_enum()?;

    let previous_end = shift_end_minute(previous.roster_day, previous_type)?;
    let current_start = shift_display_start_minute(current.roster_day, current_type);
    Some(current_start - previous_end)
}

pub fn has_insufficient_rest(
    previous: &OperatorShiftAssignment,
    current: &OperatorShiftAssignment,
) -> bool {
    previous.roster_day + 1 == current.roster_day
        && previous.is_working()
        && current.is_working()
        && shift_rest_minutes(previous, current).is_some_and(|rest| rest < MINIMUM_REST_MINUTES)
}

pub fn shift_end_minute(roster_day: RosterDay, shift_type: ShiftType) -> Option<i32> {
    let shift_id = shift_id_for_roster_day_and_type(roster_day, shift_type)?;
    Some(match shift_id {
        0 => 360,
        _ if shift_type == ShiftType::Morning => {
            (roster_day as usize * MINUTES_PER_DAY + 840) as i32
        }
        _ if shift_type == ShiftType::Afternoon => {
            (roster_day as usize * MINUTES_PER_DAY + 1320) as i32
        }
        _ if roster_day == STAFFING_ROSTER_DAY_END => CARRY_OUT_END_MINUTE as i32,
        _ => (roster_day as usize * MINUTES_PER_DAY + MINUTES_PER_DAY + 360) as i32,
    })
}
