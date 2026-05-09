use crate::domain::Room;

use super::vocabulary::room_specs;

pub(super) fn build_rooms(count: usize) -> Vec<Room> {
    room_specs()
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, spec)| Room::with_kind_capacity(index, spec.name, spec.kind, spec.capacity))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rooms() {
        let rooms = build_rooms(10);
        assert_eq!(rooms.len(), 10);
        assert_eq!(rooms[0].id, "room-0");
        assert_eq!(rooms[0].name, "Auditorium A");
        assert_eq!(rooms[7].name, "Computer Lab");
    }
}
