use chrono::{NaiveDate, Timelike};
use std::cmp::Reverse;
use std::collections::BTreeMap;

use crate::domain::{Employee, Shift};

use crate::data::data_seed::skills::is_doctor_family_skill;
use crate::data::data_seed::vocabulary::{MAX_DESIRED_DATES, MAX_UNDESIRED_DATES};

#[derive(Clone, Copy)]
pub(super) enum PreferenceKind {
    Desired,
    Undesired,
}

/// Returns whether a preference mark can be added without breaking the rules.
pub(super) fn can_mark_preference_date(
    employee: &Employee,
    date: NaiveDate,
    kind: PreferenceKind,
) -> bool {
    if employee.unavailable_dates.contains(&date) {
        return false;
    }
    match kind {
        PreferenceKind::Desired => {
            employee.desired_dates.len() < MAX_DESIRED_DATES
                && !employee.desired_dates.contains(&date)
                && !employee.undesired_dates.contains(&date)
        }
        PreferenceKind::Undesired => {
            employee.undesired_dates.len() < MAX_UNDESIRED_DATES
                && !employee.undesired_dates.contains(&date)
                && !employee.desired_dates.contains(&date)
        }
    }
}

/// Mutates the employee by adding a desired or undesired date when legal.
pub(super) fn mark_preference_date(
    employee: &mut Employee,
    date: NaiveDate,
    kind: PreferenceKind,
) -> bool {
    if !can_mark_preference_date(employee, date, kind) {
        return false;
    }
    match kind {
        PreferenceKind::Desired => employee.desired_dates.insert(date),
        PreferenceKind::Undesired => employee.undesired_dates.insert(date),
    }
}

/// Chooses the most "pressure-carrying" date touched by a shift.
pub(super) fn preferred_shift_date(
    shift: &Shift,
    date_pressure: &BTreeMap<NaiveDate, usize>,
) -> NaiveDate {
    shift
        .touched_dates
        .iter()
        .copied()
        .max_by_key(|date| (*date_pressure.get(date).unwrap_or(&0), Reverse(*date)))
        .expect("shift should touch at least one date")
}

/// Returns the precomputed pressure score for the date chosen above.
pub(super) fn date_pressure_for_shift(
    shift: &Shift,
    date_pressure: &BTreeMap<NaiveDate, usize>,
) -> usize {
    let date = preferred_shift_date(shift, date_pressure);
    *date_pressure
        .get(&date)
        .expect("preferred shift date should have a pressure score")
}

/// Small domain helper used while choosing exchange-preference targets.
pub(super) fn shift_prefers_doctor_family(shift: &Shift) -> bool {
    is_doctor_family_skill(&shift.required_skill)
}

/// Treats two shifts as the same broad "shape" for preference balancing.
pub(super) fn shift_same_shape(left: &Shift, right: &Shift) -> bool {
    left.required_skill == right.required_skill
        && (left.start.time().hour() == 22) == (right.start.time().hour() == 22)
}

#[cfg(test)]
/// Test helper for checking whether a move changes the preference score.
pub(super) fn shift_soft_preference_score(employee: &Employee, shift: &Shift) -> i64 {
    let desired_matches = employee
        .desired_dates
        .iter()
        .filter(|date| shift.touched_dates.contains(date))
        .count() as i64;
    let undesired_matches = employee
        .undesired_dates
        .iter()
        .filter(|date| shift.touched_dates.contains(date))
        .count() as i64;
    desired_matches - undesired_matches
}
