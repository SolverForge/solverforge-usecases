use crate::domain::{Lesson, Plan, Teacher};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: Teachers can only teach in slots where they are available.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    use solverforge::stream::joiner::equal_bi;

    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .join((
            ConstraintFactory::<Plan, HardMediumSoftScore>::new().for_each(Plan::teachers()),
            equal_bi(
                |lesson: &Lesson| lesson.teacher_idx,
                |teacher: &Teacher| Some(teacher.index),
            ),
        ))
        .filter(|lesson: &Lesson, teacher: &Teacher| {
            lesson.timeslot_idx.is_some_and(|timeslot_idx| {
                !teacher
                    .availability
                    .get(timeslot_idx)
                    .copied()
                    .unwrap_or(false)
            })
        })
        .penalize(HardMediumSoftScore::of_hard(1))
        .named("Teacher Availability")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Room, Timeslot, Weekday};
    use chrono::NaiveTime;
    use solverforge::ConstraintSet;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn score_for_teacher_availability(available: bool, assigned: bool) -> HardMediumSoftScore {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8), time(9))];
        let teachers = vec![Teacher::new(0, "Teacher A", vec![available])];
        let rooms = vec![Room::new(0, "Room A")];
        let mut lessons = vec![Lesson::new(0, "Math".to_string(), 0, Some(0), 60)];
        if assigned {
            lessons[0].timeslot_idx = Some(0);
            lessons[0].room_idx = Some(0);
        }
        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);

        (constraint(),).evaluate_all(&plan)
    }

    #[test]
    fn penalizes_assigned_unavailable_teacher() {
        assert_eq!(
            score_for_teacher_availability(false, true),
            HardMediumSoftScore::of_hard(-1)
        );
    }

    #[test]
    fn ignores_available_or_unassigned_teacher_slots() {
        assert_eq!(
            score_for_teacher_availability(true, true),
            HardMediumSoftScore::ZERO
        );
        assert_eq!(
            score_for_teacher_availability(false, false),
            HardMediumSoftScore::ZERO
        );
    }
}
