use crate::domain::{Lesson, Plan, Timeslot};
use chrono::{NaiveTime, Timelike};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: Prefer lessons before the late afternoon.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    use solverforge::stream::joiner::equal_bi;

    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .join((
            ConstraintFactory::<Plan, HardMediumSoftScore>::new().for_each(Plan::timeslots()),
            equal_bi(
                |lesson: &Lesson| lesson.timeslot_idx,
                |timeslot: &Timeslot| Some(timeslot.index),
            ),
        ))
        .filter(|_lesson: &Lesson, timeslot: &Timeslot| is_late_slot(timeslot.start_time))
        .penalize(HardMediumSoftScore::of_soft(1))
        .named("Avoid Late Lessons")
}

fn is_late_slot(start_time: NaiveTime) -> bool {
    start_time.hour() >= 15
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Room, Weekday};
    use solverforge::ConstraintSet;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn score_for_start_hour(hour: u32, assigned: bool) -> HardMediumSoftScore {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(hour), time(hour + 1))];
        let rooms = vec![Room::new(0, "Room A")];
        let mut lessons = vec![Lesson::new(0, "Math".to_string(), 0, None, 60)];
        if assigned {
            lessons[0].timeslot_idx = Some(0);
            lessons[0].room_idx = Some(0);
        }
        let plan = Plan::new(timeslots, vec![], vec![], lessons, rooms);

        (constraint(),).evaluate_all(&plan)
    }

    #[test]
    fn penalizes_late_assigned_lessons() {
        assert_eq!(
            score_for_start_hour(15, true),
            HardMediumSoftScore::of_soft(-1)
        );
    }

    #[test]
    fn ignores_early_or_unassigned_lessons() {
        assert_eq!(score_for_start_hour(14, true), HardMediumSoftScore::ZERO);
        assert_eq!(score_for_start_hour(15, false), HardMediumSoftScore::ZERO);
    }
}
