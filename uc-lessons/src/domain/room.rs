use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// The type of teaching space a lesson can require.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoomKind {
    #[default]
    Lecture,
    Lab,
    Computer,
    Language,
}

/// A physical teaching space available to the timetable.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Room {
    #[planning_id]
    pub id: String,
    #[serde(skip)]
    pub index: usize, // the solver-facing join key
    pub name: String,
    pub kind: RoomKind,
    pub capacity: usize,
}

impl Room {
    pub fn new(index: usize, name: impl Into<String>) -> Self {
        Self::with_kind_capacity(index, name, RoomKind::Lecture, 40)
    }

    pub fn with_kind_capacity(
        index: usize,
        name: impl Into<String>,
        kind: RoomKind,
        capacity: usize,
    ) -> Self {
        Self {
            id: format!("room-{index}"),
            index,
            name: name.into(),
            kind,
            capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_construction() {
        let fact = Room::with_kind_capacity(0, "test", RoomKind::Lecture, 40);
        assert_eq!(fact.id, "room-0");
        assert_eq!(fact.name, "test");
        assert_eq!(fact.kind, RoomKind::Lecture);
        assert_eq!(fact.capacity, 40);
    }
}
