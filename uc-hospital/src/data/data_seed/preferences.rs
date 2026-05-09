//! Preference shaping for the public dataset.
//!
//! The hidden witness gives us a hard-feasible schedule. This module then adds
//! desired/undesired dates so the public problem contains soft-score movement
//! without throwing away that feasibility margin.

mod exchange;
mod floor;
mod support;
mod top_up;

use chrono::NaiveDate;
use std::collections::{BTreeMap, BTreeSet};

use crate::domain::{Employee, Shift};

use self::exchange::assign_exchange_preferences;
use self::floor::ensure_preference_floor;
use self::top_up::{add_weekend_preference_bias, top_up_preferences};
use super::coverage::employee_can_cover_shift_without_schedule;
use super::employees::EmployeeBlueprint;
use super::vocabulary::{EXCHANGE_MARK_LIMIT_PER_EMPLOYEE, MAX_DESIRED_DATES, MAX_UNDESIRED_DATES};
use super::witness::WitnessRoster;

#[cfg(test)]
pub(super) fn shift_soft_preference_score(employee: &Employee, shift: &Shift) -> i64 {
    support::shift_soft_preference_score(employee, shift)
}

/// Adds the full preference surface for the public dataset.
pub(super) fn add_preferences(
    employees: &mut [Employee],
    start_date: NaiveDate,
    blueprints: &[EmployeeBlueprint],
    shifts: &[Shift],
    witness: &WitnessRoster,
) {
    let analysis = PreferenceAnalysis::build(employees, shifts, witness);

    // Phase 1: create direct witness-relative exchange pressure.
    //
    // For a curated subset of shifts we mark the witness holder as disliking the
    // touched date and mark one feasible alternative as preferring it. This makes
    // the hidden witness intentionally non-soft-optimal and, more importantly,
    // creates real one-move improvement opportunities in the public solution
    // space instead of generic weekday-themed noise.
    if let Some(max_exchange_marks_per_employee) = EXCHANGE_MARK_LIMIT_PER_EMPLOYEE {
        assign_exchange_preferences(
            employees,
            blueprints,
            shifts,
            witness,
            &analysis,
            max_exchange_marks_per_employee,
        );
    }

    // Phase 2: fill the remaining preference volume with stable weekday-themed
    // dates. The pure witness-relative variant created a richer soft surface,
    // but it also made cheapest-insertion too eager to burn hard feasibility.
    // The hybrid shape keeps one explicit exchange signal while leaving the
    // bulk of the dataset in a feasibility-friendly, deterministic pattern.
    top_up_preferences(employees, start_date, blueprints);

    // Phase 3: add a tiny weekend bias only when the employee is still below the
    // target preference floor. This keeps the designed feasibility margin while
    // still breaking some of the largest weekday-only symmetries.
    add_weekend_preference_bias(employees, start_date, blueprints);

    ensure_preference_floor(
        employees,
        &witness.employee_touched_dates,
        &analysis.coverable_dates_by_employee,
        &analysis.date_pressure,
    );

    for employee in employees.iter() {
        assert!(
            employee.desired_dates.len() >= 4 && employee.desired_dates.len() <= MAX_DESIRED_DATES,
            "desired-date volume should stay in the designed band"
        );
        assert!(
            employee.undesired_dates.len() >= 4
                && employee.undesired_dates.len() <= MAX_UNDESIRED_DATES,
            "undesired-date volume should stay in the designed band"
        );
        assert!(
            employee
                .desired_dates
                .iter()
                .all(|date| !employee.undesired_dates.contains(date)),
            "desired and undesired dates must stay disjoint"
        );
    }
}

/// Cached facts shared by the different preference-shaping passes.
struct PreferenceAnalysis {
    candidate_lists: Vec<Vec<usize>>,
    candidate_counts: Vec<usize>,
    witness_shifts_by_employee: Vec<Vec<usize>>,
    coverable_dates_by_employee: Vec<BTreeSet<NaiveDate>>,
    date_pressure: BTreeMap<NaiveDate, usize>,
}

impl PreferenceAnalysis {
    /// Precomputes the helper views every preference phase needs.
    fn build(employees: &[Employee], shifts: &[Shift], witness: &WitnessRoster) -> Self {
        let candidate_lists = eligible_employees_by_shift(employees, shifts);
        let candidate_counts: Vec<usize> = candidate_lists.iter().map(Vec::len).collect();
        let witness_shifts_by_employee =
            witness_shift_indices_by_employee(&witness.assignments, employees.len());
        let coverable_dates_by_employee =
            coverable_dates_by_employee(&candidate_lists, shifts, employees.len());
        let date_pressure = date_pressure_by_day(shifts, &candidate_counts);

        Self {
            candidate_lists,
            candidate_counts,
            witness_shifts_by_employee,
            coverable_dates_by_employee,
            date_pressure,
        }
    }
}

/// Lists the legal employees for each public shift before scheduling interactions.
pub(super) fn eligible_employees_by_shift(
    employees: &[Employee],
    shifts: &[Shift],
) -> Vec<Vec<usize>> {
    shifts
        .iter()
        .map(|shift| {
            employees
                .iter()
                .enumerate()
                .filter(|(_, employee)| employee_can_cover_shift_without_schedule(employee, shift))
                .map(|(employee_index, _)| employee_index)
                .collect()
        })
        .collect()
}

/// Reverses witness assignments into "which shifts belong to each employee".
fn witness_shift_indices_by_employee(
    assignments: &[usize],
    employee_count: usize,
) -> Vec<Vec<usize>> {
    let mut shifts_by_employee = vec![Vec::new(); employee_count];
    for (shift_index, &employee_index) in assignments.iter().enumerate() {
        shifts_by_employee[employee_index].push(shift_index);
    }
    shifts_by_employee
}

/// Collects every date an employee could legally cover in the public dataset.
fn coverable_dates_by_employee(
    candidate_lists: &[Vec<usize>],
    shifts: &[Shift],
    employee_count: usize,
) -> Vec<BTreeSet<NaiveDate>> {
    let mut dates_by_employee = vec![BTreeSet::new(); employee_count];
    for (shift_index, candidates) in candidate_lists.iter().enumerate() {
        for &candidate in candidates {
            dates_by_employee[candidate].extend(shifts[shift_index].touched_dates.iter().copied());
        }
    }
    dates_by_employee
}

/// Scores dates by how much soft-pressure they can safely carry.
fn date_pressure_by_day(
    shifts: &[Shift],
    candidate_counts: &[usize],
) -> BTreeMap<NaiveDate, usize> {
    let mut pressure = BTreeMap::new();
    for (shift, &candidate_count) in shifts.iter().zip(candidate_counts.iter()) {
        // The goal here is not to make the hard problem tighter. It is to create
        // lots of feasible reassignment opportunities. We therefore score dates
        // by how many "comfortable" alternatives they carry, not by how scarce
        // they are. Scarce dates belong to feasibility; abundant dates are where
        // soft pressure can live without poisoning construction.
        let shift_pressure = candidate_count.clamp(2, 8) - 1;
        for &date in &shift.touched_dates {
            *pressure.entry(date).or_default() += shift_pressure.max(1);
        }
    }
    pressure
}
