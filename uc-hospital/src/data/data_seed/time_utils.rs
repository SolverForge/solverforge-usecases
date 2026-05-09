use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::BTreeSet;

use crate::domain::Shift;

use super::vocabulary::{DAYS_IN_SCHEDULE, FIRST_NAMES, LAST_NAMES};

/// Returns the sorted set of dates touched by the published shifts.
pub(super) fn horizon_dates(shifts: &[Shift]) -> Vec<NaiveDate> {
    let mut dates = BTreeSet::new();
    for shift in shifts {
        for &date in &shift.touched_dates {
            dates.insert(date);
        }
    }
    dates.into_iter().collect()
}

/// Buckets every schedule date by weekday for preference shaping.
pub(super) fn weekday_dates(start_date: NaiveDate) -> [Vec<NaiveDate>; 7] {
    let mut dates = std::array::from_fn(|_| Vec::new());
    for day in 0..DAYS_IN_SCHEDULE {
        let date = start_date + Duration::days(day);
        dates[date.weekday().num_days_from_monday() as usize].push(date);
    }
    dates
}

/// Finds a weekday that still exposes four available dates after unavailability.
pub(super) fn choose_weekday_with_four_available_dates(
    unavailable_dates: &BTreeSet<NaiveDate>,
    primary_off: usize,
    candidates: impl IntoIterator<Item = usize>,
) -> usize {
    candidates
        .into_iter()
        .filter(|&weekday| weekday != primary_off)
        .find(|&weekday| {
            weekday_dates(find_next_monday(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ))[weekday]
                .iter()
                .filter(|date| !unavailable_dates.contains(date))
                .count()
                == 4
        })
        .expect("weekday with four available dates should exist")
}

/// Expands a time span into the calendar dates it touches.
pub(super) fn dates_touched_by_span(start: NaiveDateTime, end: NaiveDateTime) -> Vec<NaiveDate> {
    let mut touched_dates = Vec::new();
    let mut date = start.date();

    while date <= end.date() {
        if overlap_minutes_for_day(start, end, date) > 0 {
            touched_dates.push(date);
        }

        let Some(next_date) = date.succ_opt() else {
            break;
        };
        date = next_date;
    }

    touched_dates
}

/// Measures how many minutes of the span overlap one specific date.
fn overlap_minutes_for_day(start: NaiveDateTime, end: NaiveDateTime, date: NaiveDate) -> i64 {
    let day_start = date.and_hms_opt(0, 0, 0).unwrap();
    let day_end = date
        .succ_opt()
        .unwrap_or(date)
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let overlap_start = start.max(day_start);
    let overlap_end = end.min(day_end);

    if overlap_start < overlap_end {
        (overlap_end - overlap_start).num_minutes()
    } else {
        0
    }
}

/// Creates a deterministic shuffled name list for the workforce generator.
pub(super) fn generate_name_permutations(rng: &mut StdRng) -> Vec<String> {
    let mut names = Vec::with_capacity(FIRST_NAMES.len() * LAST_NAMES.len());
    for first in FIRST_NAMES {
        for last in LAST_NAMES {
            names.push(format!("{first} {last}"));
        }
    }
    names.shuffle(rng);
    names
}

/// Small helper so shift templates can read as `time(14, 0)`.
pub(super) fn time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

/// Anchors the benchmark to a Monday so weekday-based rules stay stable.
pub(super) fn find_next_monday(date: NaiveDate) -> NaiveDate {
    let days_until_monday = match date.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 6,
        Weekday::Wed => 5,
        Weekday::Thu => 4,
        Weekday::Fri => 3,
        Weekday::Sat => 2,
        Weekday::Sun => 1,
    };
    date + Duration::days(days_until_monday)
}
