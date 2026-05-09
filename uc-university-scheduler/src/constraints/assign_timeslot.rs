use crate::domain::{Lesson, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// MEDIUM: Every lesson should receive a timeslot.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .filter(|lesson: &Lesson| lesson.timeslot_idx.is_none())
        .penalize(HardMediumSoftScore::of_medium(1))
        .named("Assign Timeslot")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Room, Timeslot, Weekday};
    use chrono::NaiveTime;

    fn time(hour: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, 0, 0).unwrap()
    }

    fn plan_with_timeslot_assignment(timeslot_idx: Option<usize>) -> Plan {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8), time(9))];
        let rooms = vec![Room::new(0, "Room A")];
        let mut lessons = vec![Lesson::new(0, "Math".to_string(), 0, None, 60)];
        lessons[0].timeslot_idx = timeslot_idx;
        Plan::new(timeslots, vec![], vec![], lessons, rooms)
    }

    fn evaluate_only_assign_timeslot(plan: &Plan) -> HardMediumSoftScore {
        let constraint_set = (constraint(),);
        constraint_set.evaluate_all(plan)
    }

    #[test]
    fn penalizes_unassigned_timeslot_at_medium_level() {
        let plan = plan_with_timeslot_assignment(None);
        let score = evaluate_only_assign_timeslot(&plan);

        assert_eq!(score, HardMediumSoftScore::of_medium(-1));
    }

    #[test]
    fn assigned_timeslot_has_zero_score() {
        let plan = plan_with_timeslot_assignment(Some(0));
        let score = evaluate_only_assign_timeslot(&plan);

        assert_eq!(score, HardMediumSoftScore::ZERO);
    }
}
