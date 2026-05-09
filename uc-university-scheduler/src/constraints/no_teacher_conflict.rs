use crate::domain::{Lesson, Plan, Timeslot, Weekday};
use chrono::NaiveTime;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: No two lessons with the same teacher can overlap in time.
struct AssignedLessonSlot {
    lesson_index: usize,
    teacher_idx: usize,
    day_of_week: Weekday,
    start_time: NaiveTime,
    end_time: NaiveTime,
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
        .project(|lesson: &Lesson, timeslot: &Timeslot| AssignedLessonSlot {
            lesson_index: lesson.index,
            teacher_idx: lesson.teacher_idx.unwrap_or_else(|| usize::MAX),
            day_of_week: timeslot.day_of_week,
            start_time: timeslot.start_time,
            end_time: timeslot.end_time,
        })
        .join(equal(|row: &AssignedLessonSlot| row.teacher_idx))
        .filter(|a: &AssignedLessonSlot, b: &AssignedLessonSlot| {
            a.lesson_index < b.lesson_index
                && a.day_of_week == b.day_of_week
                && a.start_time < b.end_time
                && b.start_time < a.end_time
        })
        .penalize_hard_with(|_a: &AssignedLessonSlot, _b: &AssignedLessonSlot| {
            <HardMediumSoftScore as Score>::one_hard()
        })
        .named("No Teacher Conflict")
}

#[cfg(test)]
mod tests {
    use crate::domain::Teacher;

    use super::*;

    fn time(hour: u32, min: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, min, 0).unwrap()
    }

    // Helper that only runs the No Teacher Conflict constraint
    fn evaluate_only_teacher_conflict(plan: &Plan) -> HardMediumSoftScore {
        // Create a constraint set with only the teacher conflict constraint
        // Using a tuple to implement ConstraintSet
        let constraint_set = (constraint(),);
        constraint_set.evaluate_all(plan)
    }

    #[test]
    fn detects_teacher_conflict_same_timeslot() {
        let teachers = vec![Teacher::new(0, "Prof A", [true; 10])];
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0))];
        let rooms = vec![];
        // Different groups to avoid group conflict, no room assignment to avoid room conflict
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, Some(0), 120),
            Lesson::new(1, "Physics".to_string(), 1, Some(0), 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[1].timeslot_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_teacher_conflict(&plan);

        assert_eq!(
            score.hard(),
            -1,
            "Expected hard score -1 for teacher conflict"
        );
    }

    #[test]
    fn detects_teacher_conflict_overlapping_timeslots() {
        let teachers = vec![Teacher::new(0, "Prof A", [true; 10])];
        let timeslots = vec![
            Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0)),
            Timeslot::new(1, Weekday::Mon, time(9, 0), time(11, 0)),
        ];
        let rooms = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, Some(0), 120),
            Lesson::new(1, "Physics".to_string(), 1, Some(0), 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[1].timeslot_idx = Some(1);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_teacher_conflict(&plan);

        assert_eq!(
            score.hard(),
            -1,
            "Expected hard score -1 for overlapping teacher conflict"
        );
    }

    #[test]
    fn no_conflict_different_teachers() {
        let teachers = vec![
            Teacher::new(0, "Prof A", [true; 10]),
            Teacher::new(1, "Prof B", [true; 10]),
        ];
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0))];
        let rooms = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, Some(0), 120),
            Lesson::new(1, "Physics".to_string(), 0, Some(1), 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[1].timeslot_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_teacher_conflict(&plan);

        assert_eq!(
            score.hard(),
            0,
            "Expected hard score 0 for different teachers"
        );
    }

    #[test]
    fn no_conflict_non_overlapping_timeslots() {
        let teachers = vec![Teacher::new(0, "Prof A", [true; 10])];
        let timeslots = vec![
            Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0)),
            Timeslot::new(1, Weekday::Mon, time(10, 0), time(12, 0)),
        ];
        let rooms = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, Some(0), 120),
            Lesson::new(1, "Physics".to_string(), 0, Some(0), 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[1].timeslot_idx = Some(1);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_teacher_conflict(&plan);

        assert_eq!(
            score.hard(),
            0,
            "Expected hard score 0 for non-overlapping timeslots"
        );
    }

    #[test]
    fn no_conflict_unassigned_teacher() {
        let teachers = vec![Teacher::new(0, "Prof A", [true; 10])];
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0))];
        let rooms = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 1, Some(0), 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[1].timeslot_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_teacher_conflict(&plan);

        assert_eq!(
            score.hard(),
            0,
            "Expected hard score 0 for unassigned teacher"
        );
    }
}
