use crate::domain::{Lesson, Plan, Room};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: A room must be large enough for the cohort assigned to it.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    use solverforge::stream::joiner::equal_bi;

    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .join((
            ConstraintFactory::<Plan, HardMediumSoftScore>::new().for_each(Plan::rooms()),
            equal_bi(
                |lesson: &Lesson| lesson.room_idx,
                |room: &Room| Some(room.index),
            ),
        ))
        .filter(|lesson: &Lesson, room: &Room| lesson.student_count > room.capacity)
        .penalize(HardMediumSoftScore::of_hard(1))
        .named("Room Capacity")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Group, RoomKind, Timeslot, Weekday};
    use chrono::NaiveTime;
    use solverforge::ConstraintSet;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn score_for_room_capacity(capacity: usize, assigned: bool) -> HardMediumSoftScore {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8), time(9))];
        let groups = vec![Group::new(0, "Group A", 30, vec![true])];
        let rooms = vec![Room::with_kind_capacity(
            0,
            "Room A",
            RoomKind::Lecture,
            capacity,
        )];
        let mut lessons = vec![Lesson::new(0, "Math".to_string(), 0, None, 60)];
        if assigned {
            lessons[0].timeslot_idx = Some(0);
            lessons[0].room_idx = Some(0);
        }
        let plan = Plan::new(timeslots, vec![], groups, lessons, rooms);

        (constraint(),).evaluate_all(&plan)
    }

    #[test]
    fn penalizes_room_under_capacity() {
        assert_eq!(
            score_for_room_capacity(24, true),
            HardMediumSoftScore::of_hard(-1)
        );
    }

    #[test]
    fn ignores_sufficient_or_unassigned_room_capacity() {
        assert_eq!(
            score_for_room_capacity(30, true),
            HardMediumSoftScore::ZERO
        );
        assert_eq!(
            score_for_room_capacity(24, false),
            HardMediumSoftScore::ZERO
        );
    }
}
