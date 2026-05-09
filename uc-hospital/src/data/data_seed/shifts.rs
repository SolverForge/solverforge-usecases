use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};

use crate::domain::Shift;

use super::demand::DEMAND_RULES;
use super::time_utils::{dates_touched_by_span, find_next_monday, time};
use super::vocabulary::DAYS_IN_SCHEDULE;

/// Expands the demand template into the actual public shift entities.
pub(super) fn build_public_shifts(start_date: NaiveDate) -> Vec<Shift> {
    let mut shifts = Vec::with_capacity(expected_shift_count());
    let mut shift_id = 0usize;

    for day in 0..DAYS_IN_SCHEDULE {
        let date = start_date + Duration::days(day);
        for rule in DEMAND_RULES {
            for _ in 0..rule.count_for_date(date.weekday()) {
                let start = NaiveDateTime::new(date, time(rule.start_hour, 0));
                let end = start + Duration::hours(8);
                shifts.push(Shift::new(
                    shift_id.to_string(),
                    start,
                    end,
                    rule.location,
                    rule.required_skill,
                ));
                shift_id += 1;
            }
        }
    }

    shifts
}

/// Fills derived shift fields after the raw templates are materialized.
pub(super) fn prepare_shifts(shifts: &mut [Shift]) {
    for (index, shift) in shifts.iter_mut().enumerate() {
        shift.index = index;
        shift.touched_dates = dates_touched_by_span(shift.start, shift.end);
    }
}

/// Recomputes the expected number of public shifts from the demand template.
pub(super) fn expected_shift_count() -> usize {
    let start_date = find_next_monday(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    let mut count = 0usize;
    for day in 0..DAYS_IN_SCHEDULE {
        let date = start_date + Duration::days(day);
        for rule in DEMAND_RULES {
            count += rule.count_for_date(date.weekday());
        }
    }
    count
}
