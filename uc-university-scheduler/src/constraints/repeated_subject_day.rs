use crate::domain::{Lesson, Plan, Timeslot, Weekday};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: Prefer not to schedule the same subject twice in one day for a cohort.
struct LessonDay {
    lesson_index: usize,
    group_idx: usize,
    subject: String,
    day_of_week: Weekday,
}

pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    use solverforge::stream::joiner::{equal, equal_bi};

    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .join((
            ConstraintFactory::<Plan, HardMediumSoftScore>::new().for_each(Plan::timeslots()),
            equal_bi(
                |lesson: &Lesson| lesson.timeslot_idx,
                |timeslot: &Timeslot| Some(timeslot.index),
            ),
        ))
        .project(|lesson: &Lesson, timeslot: &Timeslot| LessonDay {
            lesson_index: lesson.index,
            group_idx: lesson.group_idx,
            subject: lesson.subject.clone(),
            day_of_week: timeslot.day_of_week,
        })
        .join(equal(|row: &LessonDay| row.group_idx))
        .filter(|a: &LessonDay, b: &LessonDay| {
            a.lesson_index < b.lesson_index
                && a.day_of_week == b.day_of_week
                && a.subject == b.subject
        })
        .penalize_with(|_a: &LessonDay, _b: &LessonDay| HardMediumSoftScore::of_soft(1))
        .named("Avoid Repeated Subject Day")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Room;
    use chrono::NaiveTime;
    use solverforge::ConstraintSet;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn score_for_subject_days(second_day: Weekday) -> HardMediumSoftScore {
        let timeslots = vec![
            Timeslot::new(0, Weekday::Mon, time(8), time(9)),
            Timeslot::new(1, second_day, time(9), time(10)),
        ];
        let rooms = vec![Room::new(0, "Room A")];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 60),
            Lesson::new(1, "Math".to_string(), 0, None, 60),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[0].room_idx = Some(0);
        lessons[1].timeslot_idx = Some(1);
        lessons[1].room_idx = Some(0);
        let plan = Plan::new(timeslots, vec![], vec![], lessons, rooms);

        (constraint(),).evaluate_all(&plan)
    }

    #[test]
    fn penalizes_repeated_subject_on_same_day() {
        assert_eq!(
            score_for_subject_days(Weekday::Mon),
            HardMediumSoftScore::of_soft(-1)
        );
    }

    #[test]
    fn ignores_same_subject_on_different_days() {
        assert_eq!(
            score_for_subject_days(Weekday::Tue),
            HardMediumSoftScore::ZERO
        );
    }
}
