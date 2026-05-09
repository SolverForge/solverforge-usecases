use crate::domain::{Lesson, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// MEDIUM: Every lesson should receive a room.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .filter(|lesson: &Lesson| lesson.room_idx.is_none())
        .penalize(HardMediumSoftScore::of_medium(1))
        .named("Assign Room")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Room, Timeslot, Weekday};
    use chrono::NaiveTime;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn plan_with_room_assignment(room_idx: Option<usize>) -> Plan {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8), time(9))];
        let rooms = vec![Room::new(0, "Room A")];
        let mut lessons = vec![Lesson::new(0, "Math".to_string(), 0, None, 60)];
        lessons[0].room_idx = room_idx;
        Plan::new(timeslots, vec![], vec![], lessons, rooms)
    }

    fn evaluate_only_assign_room(plan: &Plan) -> HardMediumSoftScore {
        let constraint_set = (constraint(),);
        constraint_set.evaluate_all(plan)
    }

    #[test]
    fn penalizes_unassigned_room_at_medium_level() {
        let plan = plan_with_room_assignment(None);
        let score = evaluate_only_assign_room(&plan);

        assert_eq!(score, HardMediumSoftScore::of_medium(-1));
    }

    #[test]
    fn assigned_room_has_zero_score() {
        let plan = plan_with_room_assignment(Some(0));
        let score = evaluate_only_assign_room(&plan);

        assert_eq!(score, HardMediumSoftScore::ZERO);
    }
}
