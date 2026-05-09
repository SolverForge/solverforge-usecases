use crate::domain::{Lesson, Plan, Timeslot, Weekday};
use chrono::NaiveTime;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: No two lessons in the same room can overlap in time.
struct AssignedLessonSlot {
    lesson_index: usize,
    room_idx: usize,
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
            room_idx: lesson.room_idx.unwrap_or_else(|| usize::MAX),
            day_of_week: timeslot.day_of_week,
            start_time: timeslot.start_time,
            end_time: timeslot.end_time,
        })
        .join(equal(|row: &AssignedLessonSlot| row.room_idx))
        .filter(|a: &AssignedLessonSlot, b: &AssignedLessonSlot| {
            a.lesson_index < b.lesson_index
                && a.day_of_week == b.day_of_week
                && a.start_time < b.end_time
                && b.start_time < a.end_time
        })
        .penalize_hard_with(|_a: &AssignedLessonSlot, _b: &AssignedLessonSlot| {
            <HardMediumSoftScore as Score>::one_hard()
        })
        .named("No Room Conflict")
}

#[cfg(test)]
mod tests {
    use crate::domain::Room;

    use super::*;

    fn time(hour: u32, min: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, min, 0).unwrap()
    }

    // Helper that only runs the No Room Conflict constraint
    fn evaluate_only_room_conflict(plan: &Plan) -> HardMediumSoftScore {
        let constraint_set = (constraint(),);
        constraint_set.evaluate_all(plan)
    }

    #[test]
    fn detects_room_conflict_same_timeslot() {
        let rooms = vec![Room::new(0, "Room A")];
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0))];
        let teachers = vec![];
        // Different groups to avoid group conflict, no teacher assignment to avoid teacher conflict
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 1, None, 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[0].room_idx = Some(0);
        lessons[1].timeslot_idx = Some(0);
        lessons[1].room_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_room_conflict(&plan);

        assert_eq!(score.hard(), -1, "Expected hard score -1 for room conflict");
    }

    #[test]
    fn detects_room_conflict_overlapping_timeslots() {
        let rooms = vec![Room::new(0, "Room A")];
        let timeslots = vec![
            Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0)),
            Timeslot::new(1, Weekday::Mon, time(9, 0), time(11, 0)),
        ];
        let teachers = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 1, None, 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[0].room_idx = Some(0);
        lessons[1].timeslot_idx = Some(1);
        lessons[1].room_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_room_conflict(&plan);

        assert_eq!(
            score.hard(),
            -1,
            "Expected hard score -1 for overlapping room conflict"
        );
    }

    #[test]
    fn no_conflict_different_rooms() {
        let rooms = vec![Room::new(0, "Room A"), Room::new(1, "Room B")];
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0))];
        let teachers = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 0, None, 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[0].room_idx = Some(0);
        lessons[1].timeslot_idx = Some(0);
        lessons[1].room_idx = Some(1);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_room_conflict(&plan);

        assert_eq!(score.hard(), 0, "Expected hard score 0 for different rooms");
    }

    #[test]
    fn no_conflict_non_overlapping_timeslots() {
        let rooms = vec![Room::new(0, "Room A")];
        let timeslots = vec![
            Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0)),
            Timeslot::new(1, Weekday::Mon, time(10, 0), time(12, 0)),
        ];
        let teachers = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 0, None, 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[0].room_idx = Some(0);
        lessons[1].timeslot_idx = Some(1);
        lessons[1].room_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_room_conflict(&plan);

        assert_eq!(
            score.hard(),
            0,
            "Expected hard score 0 for non-overlapping timeslots"
        );
    }

    #[test]
    fn no_conflict_unassigned_room() {
        let rooms = vec![Room::new(0, "Room A")];
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8, 0), time(10, 0))];
        let teachers = vec![];
        let mut lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 1, None, 120),
        ];
        lessons[0].timeslot_idx = Some(0);
        lessons[0].room_idx = None; // Unassigned
        lessons[1].timeslot_idx = Some(0);
        lessons[1].room_idx = Some(0);

        let plan = Plan::new(timeslots, teachers, vec![], lessons, rooms);
        let score = evaluate_only_room_conflict(&plan);

        assert_eq!(score.hard(), 0, "Expected hard score 0 for unassigned room");
    }
}
