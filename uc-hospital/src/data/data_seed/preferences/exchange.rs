use chrono::Timelike;
use std::cmp::Reverse;

use crate::domain::{Employee, Shift};

use super::support::{
    can_mark_preference_date, date_pressure_for_shift, mark_preference_date, preferred_shift_date,
    shift_prefers_doctor_family, shift_same_shape, PreferenceKind,
};
use super::PreferenceAnalysis;
use crate::data::data_seed::employees::EmployeeBlueprint;
use crate::data::data_seed::skills::is_specialty_skill;
use crate::data::data_seed::vocabulary::DOCTOR;
use crate::data::data_seed::witness::{shift_priority_rank, WitnessRoster};

/// Adds a small number of witness-relative preference swaps.
///
/// This is the sharpest source of local-search signal in the dataset: one
/// employee is marked as disliking a date while another feasible employee is
/// marked as preferring it.
pub(super) fn assign_exchange_preferences(
    employees: &mut [Employee],
    blueprints: &[EmployeeBlueprint],
    shifts: &[Shift],
    witness: &WitnessRoster,
    analysis: &PreferenceAnalysis,
    max_exchange_marks_per_employee: usize,
) {
    let mut exchange_marks_by_employee = vec![0usize; employees.len()];
    let mut shift_order: Vec<usize> = (0..shifts.len()).collect();
    shift_order.sort_by_key(|&shift_index| {
        let shift = &shifts[shift_index];
        (
            Reverse(analysis.candidate_counts[shift_index]),
            Reverse(date_pressure_for_shift(shift, &analysis.date_pressure)),
            shift_priority_rank(shift),
            shift.start,
            shift_index,
        )
    });

    for shift_index in shift_order {
        let shift = &shifts[shift_index];
        if analysis.candidate_counts[shift_index] < 6
            || is_specialty_skill(&shift.required_skill)
            || shift.start.time().hour() == 22
        {
            continue;
        }
        let holder = witness.assignments[shift_index];
        let date = preferred_shift_date(shift, &analysis.date_pressure);

        let Some(alternative) = analysis.candidate_lists[shift_index]
            .iter()
            .copied()
            .filter(|&candidate| candidate != holder)
            .filter(|&candidate| {
                exchange_marks_by_employee[candidate] < max_exchange_marks_per_employee
            })
            .filter(|&candidate| {
                can_mark_preference_date(&employees[candidate], date, PreferenceKind::Desired)
            })
            .min_by_key(|&candidate| {
                exchange_alternative_key(candidate, shift, shifts, blueprints, witness, analysis)
            })
        else {
            continue;
        };

        if mark_preference_date(&mut employees[alternative], date, PreferenceKind::Desired) {
            exchange_marks_by_employee[alternative] += 1;
        }
    }
}

/// Lower keys mean "better alternative employee for this exchange mark".
fn exchange_alternative_key(
    candidate: usize,
    shift: &Shift,
    shifts: &[Shift],
    blueprints: &[EmployeeBlueprint],
    witness: &WitnessRoster,
    analysis: &PreferenceAnalysis,
) -> (usize, usize, usize, usize, usize, usize) {
    let date = preferred_shift_date(shift, &analysis.date_pressure);
    let same_date_in_witness =
        usize::from(witness.employee_touched_dates[candidate].contains(&date));
    let same_shape_load = analysis.witness_shifts_by_employee[candidate]
        .iter()
        .filter(|&&other_shift_index| shift_same_shape(shift, &shifts[other_shift_index]))
        .count();
    (
        same_date_in_witness,
        usize::from(
            blueprints[candidate].skills.contains(DOCTOR) != shift_prefers_doctor_family(shift),
        ),
        same_shape_load,
        witness.employee_touched_dates[candidate].len(),
        usize::MAX - *analysis.date_pressure.get(&date).unwrap_or(&0),
        candidate,
    )
}
