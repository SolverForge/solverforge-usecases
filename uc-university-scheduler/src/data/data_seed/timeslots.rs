use chrono::NaiveTime;

use crate::domain::{Timeslot, Weekday};

use super::vocabulary::{
    DAY_END_HOUR, DAY_START_HOUR, LESSON_DURATION_HOURS, LUNCH_BREAK_END, LUNCH_BREAK_START,
};

/// Builds a deterministic set of timeslots for the schedule.
///
/// Creates 1-hour timeslots for each day of the week (Monday-Friday):
/// - 8:00-9:00
/// - 9:00-10:00
/// - 10:00-11:00
/// - 11:00-12:00
/// - 14:00-15:00  (lunch break 12:00-14:00 skipped)
/// - 15:00-16:00
/// - 16:00-17:00
/// - 17:00-18:00
///
/// Total: 40 timeslots (5 days * 8 slots per day)
pub(super) fn build_timeslots(count: usize) -> Vec<Timeslot> {
    // If count is 0 or very small, use the simple sequential approach
    if count <= 7 {
        return build_simple_timeslots(count);
    }

    // Build 1-hour timeslots for a full week, skipping lunch break (12:00-14:00)
    let mut timeslots = Vec::with_capacity(count.min(40));
    let mut index = 0;

    for day in [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ] {
        if index >= count {
            break;
        }

        // Generate hour slots from 8:00 to 18:00, skipping 12:00-14:00
        for hour in DAY_START_HOUR..DAY_END_HOUR {
            if index >= count {
                break;
            }

            // Skip lunch break: no timeslots from 12:00 to 14:00
            if hour >= LUNCH_BREAK_START && hour < LUNCH_BREAK_END {
                continue;
            }

            let start_hour = hour;
            let end_hour = start_hour + LESSON_DURATION_HOURS;

            timeslots.push(Timeslot::new(
                index,
                day,
                NaiveTime::from_hms_opt(start_hour, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(end_hour, 0, 0).unwrap(),
            ));
            index += 1;
        }
    }

    timeslots
}

/// Builds simple sequential timeslots with default values.
/// Used for small test cases or when count is very small.
pub(super) fn build_simple_timeslots(count: usize) -> Vec<Timeslot> {
    (0..count)
        .map(|index| Timeslot {
            id: format!("timeslot-{index}"),
            index,
            day_of_week: Default::default(),
            start_time: Default::default(),
            end_time: Default::default(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;

    #[test]
    fn test_build_simple_timeslots() {
        let timeslots = build_simple_timeslots(3);
        assert_eq!(timeslots.len(), 3);
        assert_eq!(timeslots[0].id, "timeslot-0");
        assert_eq!(timeslots[1].id, "timeslot-1");
        assert_eq!(timeslots[2].id, "timeslot-2");
    }

    #[test]
    fn test_build_timeslots_full_week() {
        let timeslots = build_timeslots(40);
        assert_eq!(timeslots.len(), 40);

        // Check first timeslot is Monday 8:00-9:00
        assert_eq!(timeslots[0].index, 0);
        assert_eq!(timeslots[0].day_of_week, Weekday::Mon);
        assert_eq!(
            timeslots[0].start_time,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[0].end_time,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );

        // Check Monday morning slots (8:00-9:00, 9:00-10:00, 10:00-11:00, 11:00-12:00)
        assert_eq!(
            timeslots[0].start_time,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[1].start_time,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[2].start_time,
            NaiveTime::from_hms_opt(10, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[3].start_time,
            NaiveTime::from_hms_opt(11, 0, 0).unwrap()
        );

        // Check that 12:00-13:00 and 13:00-14:00 are skipped (no slot starts at 12 or 13)
        // After 11:00-12:00 (index 3), next should be 14:00-15:00
        assert_eq!(
            timeslots[4].start_time,
            NaiveTime::from_hms_opt(14, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[4].end_time,
            NaiveTime::from_hms_opt(15, 0, 0).unwrap()
        );

        // Check Monday afternoon slots (14:00-15:00, 15:00-16:00, 16:00-17:00, 17:00-18:00)
        assert_eq!(
            timeslots[4].start_time,
            NaiveTime::from_hms_opt(14, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[5].start_time,
            NaiveTime::from_hms_opt(15, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[6].start_time,
            NaiveTime::from_hms_opt(16, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[7].start_time,
            NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        );

        // Check first timeslot of Tuesday (index 8)
        assert_eq!(timeslots[8].index, 8);
        assert_eq!(timeslots[8].day_of_week, Weekday::Tue);
        assert_eq!(
            timeslots[8].start_time,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_build_timeslots_partial() {
        let timeslots = build_timeslots(10);
        assert_eq!(timeslots.len(), 10);
        // Should have Monday (8 slots) + first 2 of Tuesday = 10 timeslots
        assert_eq!(timeslots[0].day_of_week, Weekday::Mon);
        assert_eq!(timeslots[8].day_of_week, Weekday::Tue);
        assert_eq!(
            timeslots[8].start_time,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            timeslots[9].start_time,
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_no_lunch_break_slots() {
        let timeslots = build_timeslots(40);
        // Verify no timeslot starts at 12:00 or 13:00
        for timeslot in &timeslots {
            let start_hour = timeslot.start_time.hour();
            assert_ne!(start_hour, 12, "No timeslot should start at 12:00");
            assert_ne!(start_hour, 13, "No timeslot should start at 13:00");
        }
    }

    #[test]
    fn test_all_slots_are_one_hour() {
        let timeslots = build_timeslots(40);
        for timeslot in &timeslots {
            let duration = timeslot.end_time.signed_duration_since(timeslot.start_time);
            assert_eq!(duration.num_hours(), 1, "All timeslots should be 1 hour");
        }
    }
}
