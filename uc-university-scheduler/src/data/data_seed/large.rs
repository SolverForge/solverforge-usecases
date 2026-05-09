use std::sync::OnceLock;

use crate::domain::Plan;

use super::groups::build_groups;
use super::lessons::build_lessons;
use super::rooms::build_rooms;
use super::teachers::build_teachers;
use super::timeslots::build_timeslots;
use super::vocabulary::{GROUP_COUNT, ROOM_COUNT, TIMESLOT_COUNT};

/// Materializes the canonical university benchmark dataset.
///
/// We cache the built plan because demo data is immutable and deterministic.
/// Reusing the same constructed instance avoids paying generator cost on every
/// API request while still returning an owned `Plan` to each caller.
pub fn generate_large() -> Plan {
    static SCHEDULE: OnceLock<Plan> = OnceLock::new();
    SCHEDULE.get_or_init(build_large_schedule).clone()
}

/// Builds the large university timetable instance from scratch.
///
/// This generates a substantial dataset suitable for benchmarking:
/// - 40 timeslots (full week: Monday-Friday, 8:00-18:00, skipping lunch 12:00-14:00)
/// - 20 teachers with subject-specific availability
/// - 12 groups
/// - 300 lessons (25 per group based on subject hours allocation)
/// - 10 typed rooms
fn build_large_schedule() -> Plan {
    // Full week timeslots: 5 days * 8 slots per day = 40 timeslots
    let timeslots = build_timeslots(TIMESLOT_COUNT);

    let teachers = build_teachers();

    let groups = build_groups(GROUP_COUNT);

    let lessons = build_lessons(GROUP_COUNT);

    let rooms = build_rooms(ROOM_COUNT);

    Plan::new(timeslots, teachers, groups, lessons, rooms)
}
