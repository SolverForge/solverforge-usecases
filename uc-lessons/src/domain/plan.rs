use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::Group;
use super::Lesson;
use super::Teacher;
use super::Timeslot;
use super::Room;
// @solverforge:end solution-imports

/// The root planning solution.
///
/// Fresh projects start as a neutral shell. Add fact collections, planning
/// entity collections, and variable fields through the CLI as your domain
/// takes shape.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[derive(Serialize, Deserialize)]
pub struct Plan {
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub timeslots: Vec<Timeslot>,
    #[problem_fact_collection]
    pub teachers: Vec<Teacher>,
    #[problem_fact_collection]
    pub groups: Vec<Group>,
    #[planning_entity_collection]
    pub lessons: Vec<Lesson>,
    #[problem_fact_collection]
    pub rooms: Vec<Room>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardMediumSoftScore>,
}

impl Plan {
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        timeslots: Vec<Timeslot>,
        teachers: Vec<Teacher>,
        groups: Vec<Group>,
        lessons: Vec<Lesson>,
        rooms: Vec<Room>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        let mut shedule: Plan = Self{
            // @solverforge:begin solution-constructor-init
            timeslots,
            teachers,
            groups,
            lessons,
            rooms,
            // @solverforge:end solution-constructor-init
            score: None,
        };
        shedule.rebuild_derived_fields();
        shedule
    }

    /// Recomputes indexes for entity join keys.
    ///
    /// This runs after generation and after transport decoding so the domain
    /// model always reaches the solver in a normalized state.
    ///
    /// Sets the `index` field on facts and lessons to match their position in
    /// their respective collections. These indexes are used as solver-facing
    /// join keys for constraint streams (e.g., `lesson.timeslot_idx` joins with
    /// `timeslot.index`, while `lesson.index` separates lesson pairs).
    pub fn rebuild_derived_fields(&mut self) {
        for (index, timeslot) in self.timeslots.iter_mut().enumerate() {
            timeslot.index = index;
        }
        for (index, teacher) in self.teachers.iter_mut().enumerate() {
            teacher.index = index;
        }
        for (index, group) in self.groups.iter_mut().enumerate() {
            group.index = index;
        }
        for (index, room) in self.rooms.iter_mut().enumerate() {
            room.index = index;
        }

        // Validate planning variable assignments to ensure they remain within
        // bounds after deserialization or data generation.
        for (index, lesson) in self.lessons.iter_mut().enumerate() {
            lesson.index = index;
            lesson.timeslot_idx = lesson
                .timeslot_idx
                .filter(|idx| *idx < self.timeslots.len());
            lesson.room_idx = lesson.room_idx.filter(|idx| *idx < self.rooms.len());
        }
    }

    /// Safe index lookup used by constraints and diagnostics.
    #[inline]
    pub fn get_timeslot(&self, idx: usize) -> Option<&Timeslot> {
        self.timeslots.get(idx)
    }

    /// Safe index lookup used by constraints and diagnostics.
    #[inline]
    pub fn get_teacher(&self, idx: usize) -> Option<&Teacher> {
        self.teachers.get(idx)
    }

    /// Safe index lookup used by constraints and diagnostics.
    #[inline]
    pub fn get_group(&self, idx: usize) -> Option<&Group> {
        self.groups.get(idx)
    }

    /// Safe index lookup used by constraints and diagnostics.
    #[inline]
    pub fn get_room(&self, idx: usize) -> Option<&Room> {
        self.rooms.get(idx)
    }

    /// Named slice accessor used by joins and generated transport code.
    #[inline]
    pub fn timeslots_slice(&self) -> &[Timeslot] {
        self.timeslots.as_slice()
    }

    /// Named slice accessor used by joins and generated transport code.
    #[inline]
    pub fn teachers_slice(&self) -> &[Teacher] {
        self.teachers.as_slice()
    }

    /// Named slice accessor used by joins and generated transport code.
    #[inline]
    pub fn groups_slice(&self) -> &[Group] {
        self.groups.as_slice()
    }

    /// Named slice accessor used by joins and generated transport code.
    #[inline]
    pub fn rooms_slice(&self) -> &[Room] {
        self.rooms.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Weekday;

    #[test]
    fn test_rebuild_derived_fields_filters_out_of_bounds_indices() {
        use chrono::NaiveTime;

        // Create a plan with 2 timeslots and 2 rooms
        let timeslots = vec![
            Timeslot::new(0, Weekday::Mon, NaiveTime::from_hms_opt(8, 0, 0).unwrap(), NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            Timeslot::new(1, Weekday::Mon, NaiveTime::from_hms_opt(10, 0, 0).unwrap(), NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
        ];
        let teachers = vec![];
        let groups = vec![];
        let rooms = vec![
            Room::new(0, "Room A"),
            Room::new(1, "Room B"),
        ];
        let lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 0, None, 120),
        ];

        let mut plan = Plan::new(timeslots, teachers, groups, lessons, rooms);

        // Manually corrupt the indices to simulate deserialization from invalid data
        plan.lessons[0].timeslot_idx = Some(100); // Out of bounds
        plan.lessons[0].room_idx = Some(100);    // Out of bounds
        plan.lessons[1].timeslot_idx = Some(1);  // Valid
        plan.lessons[1].room_idx = Some(1);      // Valid

        // Rebuild should filter out the invalid indices
        plan.rebuild_derived_fields();

        // timeslot_idx=100 should be filtered to None (only 2 timeslots exist)
        assert_eq!(plan.lessons[0].timeslot_idx, None);
        // room_idx=100 should be filtered to None (only 2 rooms exist)
        assert_eq!(plan.lessons[0].room_idx, None);
        // Valid indices should remain
        assert_eq!(plan.lessons[1].timeslot_idx, Some(1));
        assert_eq!(plan.lessons[1].room_idx, Some(1));
    }

    #[test]
    fn test_rebuild_derived_fields_restores_lesson_indexes() {
        use chrono::NaiveTime;

        let timeslots = vec![Timeslot::new(
            0,
            Weekday::Mon,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )];
        let lessons = vec![
            Lesson::new(0, "Math".to_string(), 0, None, 120),
            Lesson::new(1, "Physics".to_string(), 0, None, 120),
        ];
        let mut plan = Plan::new(timeslots, vec![], vec![], lessons, vec![]);

        plan.lessons[0].index = 0;
        plan.lessons[1].index = 0;
        plan.rebuild_derived_fields();

        assert_eq!(plan.lessons[0].index, 0);
        assert_eq!(plan.lessons[1].index, 1);
    }

    #[test]
    fn test_getters_return_none_for_invalid_indices() {
        use chrono::NaiveTime;

        let timeslots = vec![Timeslot::new(0, Weekday::Mon, NaiveTime::from_hms_opt(8, 0, 0).unwrap(), NaiveTime::from_hms_opt(10, 0, 0).unwrap())];
        let teachers = vec![Teacher::new(0, "Teacher A", [true; 10])];
        let groups = vec![Group::new(0, "Group A", 30, [true; 10])];
        let rooms = vec![Room::new(0, "Room A")];
        let lessons = vec![];

        let plan = Plan::new(timeslots, teachers, groups, lessons, rooms);

        // Valid indices
        assert!(plan.get_timeslot(0).is_some());
        assert!(plan.get_teacher(0).is_some());
        assert!(plan.get_group(0).is_some());
        assert!(plan.get_room(0).is_some());

        // Out of bounds indices
        assert!(plan.get_timeslot(100).is_none());
        assert!(plan.get_teacher(100).is_none());
        assert!(plan.get_group(100).is_none());
        assert!(plan.get_room(100).is_none());
    }
}
