use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

use super::RoomKind;

/// A subject meeting that the solver assigns to one timeslot and one room.
#[planning_entity]
#[derive(Serialize, Deserialize)]
pub struct Lesson {
    #[planning_id]
    pub id: String,
    #[serde(skip)]
    pub index: usize, // the solver-facing join key
    pub subject: String,
    pub group_idx: usize,
    pub student_count: usize,
    pub teacher_idx: Option<usize>,
    pub duration: u32,
    pub required_room_kind: RoomKind,
    // @solverforge:begin entity-variables
    #[planning_variable(value_range_provider = "timeslots", allows_unassigned = false)]
    pub timeslot_idx: Option<usize>,
    #[planning_variable(value_range_provider = "rooms", allows_unassigned = false)]
    pub room_idx: Option<usize>,
    // @solverforge:end entity-variables
}

impl Lesson {
    pub fn new(
        index: usize,
        subject: String,
        group_idx: usize,
        teacher_idx: Option<usize>,
        duration: u32,
    ) -> Self {
        Self::with_required_room_kind(
            index,
            subject,
            group_idx,
            teacher_idx,
            duration,
            RoomKind::Lecture,
        )
    }

    pub fn with_required_room_kind(
        index: usize,
        subject: String,
        group_idx: usize,
        teacher_idx: Option<usize>,
        duration: u32,
        required_room_kind: RoomKind,
    ) -> Self {
        Self::with_details(
            index,
            subject,
            group_idx,
            30,
            teacher_idx,
            duration,
            required_room_kind,
        )
    }

    pub fn with_details(
        index: usize,
        subject: String,
        group_idx: usize,
        student_count: usize,
        teacher_idx: Option<usize>,
        duration: u32,
        required_room_kind: RoomKind,
    ) -> Self {
        Self {
            id: format!("lesson-{index}"),
            index,
            subject,
            group_idx,
            student_count,
            teacher_idx,
            duration,
            required_room_kind,
            // @solverforge:begin entity-variable-init
            timeslot_idx: None,
            room_idx: None,
            // @solverforge:end entity-variable-init
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lesson_construction() {
        let entity = Lesson::new(
            0,
            "test".to_string(),
            Default::default(),
            None,
            Default::default(),
        );
        assert_eq!(entity.id, "lesson-0");
        let _ = &entity.subject;
        let _ = &entity.group_idx;
        let _ = &entity.student_count;
        let _ = &entity.teacher_idx;
        let _ = &entity.duration;
        let _ = &entity.required_room_kind;
    }
}
