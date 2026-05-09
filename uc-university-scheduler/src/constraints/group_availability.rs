use crate::domain::{Group, Lesson, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: Cohorts can only attend lessons in slots where they are available.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    use solverforge::stream::joiner::equal_bi;

    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::lessons())
        .join((
            ConstraintFactory::<Plan, HardMediumSoftScore>::new().for_each(Plan::groups()),
            equal_bi(
                |lesson: &Lesson| lesson.group_idx,
                |group: &Group| group.index,
            ),
        ))
        .filter(|lesson: &Lesson, group: &Group| {
            lesson.timeslot_idx.is_some_and(|timeslot_idx| {
                !group
                    .availability
                    .get(timeslot_idx)
                    .copied()
                    .unwrap_or(false)
            })
        })
        .penalize(HardMediumSoftScore::of_hard(1))
        .named("Group Availability")
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

    fn score_for_group_availability(available: bool, assigned: bool) -> HardMediumSoftScore {
        let timeslots = vec![Timeslot::new(0, Weekday::Mon, time(8), time(9))];
        let groups = vec![Group::new(0, "Group A", 30, vec![available])];
        let rooms = vec![Room::new(0, "Room A")];
        let mut lessons = vec![Lesson::new(0, "Math".to_string(), 0, None, 60)];
        if assigned {
            lessons[0].timeslot_idx = Some(0);
            lessons[0].room_idx = Some(0);
        }
        let plan = Plan::new(timeslots, vec![], groups, lessons, rooms);

        (constraint(),).evaluate_all(&plan)
    }

    #[test]
    fn penalizes_assigned_unavailable_group() {
        assert_eq!(
            score_for_group_availability(false, true),
            HardMediumSoftScore::of_hard(-1)
        );
    }

    #[test]
    fn ignores_available_or_unassigned_group_slots() {
        assert_eq!(
            score_for_group_availability(true, true),
            HardMediumSoftScore::ZERO
        );
        assert_eq!(
            score_for_group_availability(false, false),
            HardMediumSoftScore::ZERO
        );
    }
}
