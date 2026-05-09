use crate::domain::{Lesson, Plan, Room};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: Prefer assigning lessons to rooms that support the subject.
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
        .filter(|lesson: &Lesson, room: &Room| lesson.required_room_kind != room.kind)
        .penalize(HardMediumSoftScore::of_soft(1))
        .named("Room Kind")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RoomKind, Timeslot, Weekday};
    use chrono::NaiveTime;
    use solverforge::ConstraintSet;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn score_for_room_kind(room_kind: RoomKind, assigned: bool) -> HardMediumSoftScore {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8), time(9))];
        let rooms = vec![Room::with_kind_capacity(0, "Room A", room_kind, 40)];
        let mut lessons = vec![Lesson::with_required_room_kind(
            0,
            "Chemistry".to_string(),
            0,
            None,
            60,
            RoomKind::Lab,
        )];
        if assigned {
            lessons[0].timeslot_idx = Some(0);
            lessons[0].room_idx = Some(0);
        }
        let plan = Plan::new(timeslots, vec![], vec![], lessons, rooms);

        (constraint(),).evaluate_all(&plan)
    }

    #[test]
    fn penalizes_room_kind_mismatch() {
        assert_eq!(
            score_for_room_kind(RoomKind::Lecture, true),
            HardMediumSoftScore::of_soft(-1)
        );
    }

    #[test]
    fn ignores_matching_or_unassigned_rooms() {
        assert_eq!(
            score_for_room_kind(RoomKind::Lab, true),
            HardMediumSoftScore::ZERO
        );
        assert_eq!(
            score_for_room_kind(RoomKind::Lecture, false),
            HardMediumSoftScore::ZERO
        );
    }
}
