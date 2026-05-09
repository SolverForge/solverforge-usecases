use chrono::NaiveDate;
use std::cmp::Reverse;
use std::collections::BTreeSet;

use crate::domain::{Employee, Shift};

use super::coverage::{candidate_redundancy_is_valid, public_candidate_counts};
use super::time_utils::horizon_dates;
use super::vocabulary::EXTRA_UNAVAILABLE_COUNT;

/// Adds a small amount of extra unavailability without breaking public feasibility.
///
/// The goal is not to make the dataset impossible. The goal is to remove some
/// trivial interchangeable assignments so local search has a clearer signal.
pub(super) fn add_extra_unavailability(
    employees: &mut [Employee],
    shifts: &[Shift],
    witness_dates: &[BTreeSet<NaiveDate>],
) {
    let horizon_dates = horizon_dates(shifts);

    for _ in 0..EXTRA_UNAVAILABLE_COUNT {
        let best_candidate = (0..employees.len())
            .flat_map(|employee_index| {
                horizon_dates
                    .iter()
                    .copied()
                    .map(move |date| (employee_index, date))
            })
            .filter(|&(employee_index, date)| {
                !employees[employee_index].unavailable_dates.contains(&date)
                    && !witness_dates[employee_index].contains(&date)
            })
            .filter_map(|(employee_index, date)| {
                let score = extra_unavailability_score(employees, shifts, employee_index, date)?;
                Some((score, employee_index, date))
            })
            .max_by_key(|&(score, employee_index, date)| {
                (score, Reverse(employee_index), Reverse(date))
            });

        let Some((_, employee_index, date)) = best_candidate else {
            break;
        };
        employees[employee_index].unavailable_dates.insert(date);
    }
}

/// Scores one candidate "employee unavailable on date" mutation.
fn extra_unavailability_score(
    employees: &[Employee],
    shifts: &[Shift],
    employee_index: usize,
    date: NaiveDate,
) -> Option<(usize, usize, usize)> {
    let mut cloned: Vec<Employee> = employees.to_vec();
    cloned[employee_index].unavailable_dates.insert(date);
    if !candidate_redundancy_is_valid(&cloned, shifts) {
        return None;
    }

    let counts = public_candidate_counts(&cloned, shifts);
    let affected: Vec<usize> = shifts
        .iter()
        .enumerate()
        .filter(|(_, shift)| shift.touched_dates.contains(&date))
        .map(|(index, _)| counts[index])
        .collect();
    let min_affected = affected.into_iter().min().unwrap_or(usize::MAX);
    let shifts_with_three_plus = counts.iter().filter(|&&count| count >= 3).count();
    Some((min_affected, shifts_with_three_plus, counts.iter().sum()))
}
