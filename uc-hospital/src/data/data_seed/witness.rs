use chrono::{NaiveDate, Timelike};
use std::collections::BTreeSet;

use crate::domain::{Employee, Shift};

use super::coverage::employee_can_cover_shift_without_schedule;
use super::skills::{is_doctor_family_skill, is_nurse_family_skill, is_specialty_skill};
use super::vocabulary::*;

/// Running load summary while building the hidden feasible witness roster.
#[derive(Default)]
struct WitnessLoad {
    shift_indices: Vec<usize>,
    touched_date_load: usize,
    night_count: usize,
    specialty_count: usize,
}

/// Internal, never-exposed feasible assignment used to shape the public dataset.
pub(super) struct WitnessRoster {
    pub(super) assignments: Vec<usize>,
    pub(super) employee_touched_dates: Vec<BTreeSet<NaiveDate>>,
}

/// Builds a guaranteed hard-feasible hidden roster for the generated shifts.
pub(super) fn build_hidden_witness(employees: &[Employee], shifts: &[Shift]) -> WitnessRoster {
    let mut assignments = vec![usize::MAX; shifts.len()];
    let mut loads: Vec<WitnessLoad> = (0..employees.len())
        .map(|_| WitnessLoad::default())
        .collect();
    let mut employee_touched_dates: Vec<BTreeSet<NaiveDate>> =
        (0..employees.len()).map(|_| BTreeSet::new()).collect();
    let eligible_employees_by_shift: Vec<Vec<usize>> = shifts
        .iter()
        .map(|shift| {
            employees
                .iter()
                .enumerate()
                .filter(|(_, employee)| employee_can_cover_shift_without_schedule(employee, shift))
                .map(|(employee_index, _)| employee_index)
                .collect()
        })
        .collect();
    let mut remaining: Vec<usize> = (0..shifts.len()).collect();

    while !remaining.is_empty() {
        let (remaining_index, shift_index, feasible_employees) = remaining
            .iter()
            .enumerate()
            .map(|(remaining_index, &shift_index)| {
                let shift = &shifts[shift_index];
                let feasible_employees: Vec<usize> = eligible_employees_by_shift[shift_index]
                    .iter()
                    .copied()
                    .filter(|&employee_index| {
                        employee_can_cover_shift_in_witness(
                            employee_index,
                            shift,
                            employees,
                            shifts,
                            &loads,
                        )
                    })
                    .collect();
                (remaining_index, shift_index, feasible_employees)
            })
            .min_by_key(|(_, shift_index, feasible_employees)| {
                let shift = &shifts[*shift_index];
                (
                    feasible_employees.len(),
                    eligible_employees_by_shift[*shift_index].len(),
                    shift_priority_rank(shift),
                    shift.start,
                    *shift_index,
                )
            })
            .expect("remaining shifts should produce a selection");

        let shift = &shifts[shift_index];

        assert!(
            !feasible_employees.is_empty(),
            "witness roster should be feasible for shift {} {} {}",
            shift.id,
            shift.location,
            shift.required_skill
        );

        let employee_index = feasible_employees
            .into_iter()
            .min_by_key(|&candidate| witness_candidate_key(candidate, shift, employees, &loads))
            .expect("feasible employee should exist");

        assignments[shift_index] = employee_index;
        let load = &mut loads[employee_index];
        load.shift_indices.push(shift_index);
        load.touched_date_load += shift.touched_dates.len();
        if shift.start.time().hour() == 22 {
            load.night_count += 1;
        }
        if is_specialty_skill(&shift.required_skill) {
            load.specialty_count += 1;
        }
        employee_touched_dates[employee_index].extend(shift.touched_dates.iter().copied());
        remaining.swap_remove(remaining_index);
    }

    WitnessRoster {
        assignments,
        employee_touched_dates,
    }
}

/// Orders shifts from hardest-to-place to easiest-to-place for witness construction.
pub(super) fn shift_priority_rank(shift: &Shift) -> usize {
    match (shift.required_skill.as_str(), shift.start.time().hour()) {
        (CARDIOLOGY, _) => 0,
        (ANAESTHETICS, _) => 1,
        (RADIOLOGY_CALL, _) => 2,
        (RADIOLOGY_DAY, _) => 3,
        (skill, 22) if is_doctor_family_skill(skill) => 4,
        (skill, _) if is_doctor_family_skill(skill) => 5,
        (skill, 22) if is_nurse_family_skill(skill) => 6,
        (skill, _) if is_nurse_family_skill(skill) => 7,
        _ => 8,
    }
}

/// Lower keys mean "better witness assignee for this shift".
fn witness_candidate_key(
    employee_index: usize,
    shift: &Shift,
    employees: &[Employee],
    loads: &[WitnessLoad],
) -> (usize, usize, usize, usize, usize, usize, usize) {
    let load = &loads[employee_index];
    let employee = &employees[employee_index];
    (
        witness_role_mismatch_penalty(employee, shift),
        witness_specialty_overhang(employee, shift),
        load.touched_date_load,
        load.night_count,
        load.specialty_count,
        witness_is_floater(employee),
        employee.index,
    )
}

/// Penalizes choosing a doctor-family mismatch or nurse-family mismatch.
fn witness_role_mismatch_penalty(employee: &Employee, shift: &Shift) -> usize {
    let has_doctor = employee.skills.contains(DOCTOR);
    let has_nurse = employee.skills.contains(NURSE);
    if is_doctor_family_skill(&shift.required_skill) {
        usize::from(!has_doctor) * 10
    } else if is_nurse_family_skill(&shift.required_skill) {
        usize::from(!has_nurse) * 10
    } else {
        0
    }
}

/// Prefers not to spend scarce specialties on non-matching work when avoidable.
fn witness_specialty_overhang(employee: &Employee, shift: &Shift) -> usize {
    [CARDIOLOGY, ANAESTHETICS, RADIOLOGY_CALL, RADIOLOGY_DAY]
        .into_iter()
        .filter(|skill| employee.skills.contains(*skill) && *skill != shift.required_skill)
        .count()
}

/// Detects the rare "super-floater" profile so it is used carefully.
fn witness_is_floater(employee: &Employee) -> usize {
    usize::from(
        employee.skills.contains(CARDIOLOGY)
            && employee.skills.contains(ANAESTHETICS)
            && employee.skills.contains(RADIOLOGY_CALL),
    )
}

/// Checks whether one employee can take a shift inside the hidden witness roster.
fn employee_can_cover_shift_in_witness(
    employee_index: usize,
    shift: &Shift,
    employees: &[Employee],
    shifts: &[Shift],
    loads: &[WitnessLoad],
) -> bool {
    let employee = &employees[employee_index];
    if !employee.skills.contains(&shift.required_skill) {
        return false;
    }
    if shift
        .touched_dates
        .iter()
        .any(|date| employee.unavailable_dates.contains(date))
    {
        return false;
    }

    for &other_shift_index in &loads[employee_index].shift_indices {
        let other = &shifts[other_shift_index];
        if shares_touched_date(shift, other)
            || shifts_overlap(shift, other)
            || violates_rest(shift, other)
        {
            return false;
        }
    }

    true
}

/// Returns whether two shifts touch any common calendar date.
fn shares_touched_date(left: &Shift, right: &Shift) -> bool {
    left.touched_dates
        .iter()
        .any(|date| right.touched_dates.contains(date))
}

/// Returns whether two shift time windows overlap in absolute time.
fn shifts_overlap(left: &Shift, right: &Shift) -> bool {
    left.start < right.end && right.start < left.end
}

/// Returns whether two non-overlapping shifts still violate the 10-hour rest rule.
fn violates_rest(left: &Shift, right: &Shift) -> bool {
    let (earlier, later) = if left.end <= right.start {
        (left, right)
    } else if right.end <= left.start {
        (right, left)
    } else {
        return false;
    };
    let gap_minutes = (later.start - earlier.end).num_minutes();
    (0..600).contains(&gap_minutes)
}
