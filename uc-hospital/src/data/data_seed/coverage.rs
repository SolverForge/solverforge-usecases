use chrono::Timelike;

use crate::domain::{Employee, Shift};

use super::skills::is_specialty_skill;

/// Checks that the public dataset still has enough legal candidates per shift.
pub(super) fn candidate_redundancy_is_valid(employees: &[Employee], shifts: &[Shift]) -> bool {
    let counts = public_candidate_counts(employees, shifts);
    if counts.iter().any(|&count| count < 2) {
        return false;
    }
    if counts.iter().filter(|&&count| count >= 3).count() * 4 < shifts.len() {
        return false;
    }
    if shifts.iter().zip(counts.iter()).any(|(shift, &count)| {
        is_specialty_skill(&shift.required_skill) && shift.start.time().hour() == 22 && count < 2
    }) {
        return false;
    }
    true
}

/// Counts legal candidates per shift without considering schedule interactions.
pub(super) fn public_candidate_counts(employees: &[Employee], shifts: &[Shift]) -> Vec<usize> {
    shifts
        .iter()
        .map(|shift| {
            employees
                .iter()
                .filter(|employee| employee_can_cover_shift_without_schedule(employee, shift))
                .count()
        })
        .collect()
}

/// Coarse feasibility check used during dataset shaping.
pub(super) fn employee_can_cover_shift_without_schedule(
    employee: &Employee,
    shift: &Shift,
) -> bool {
    employee.skills.contains(&shift.required_skill)
        && shift
            .touched_dates
            .iter()
            .all(|date| !employee.unavailable_dates.contains(date))
}
