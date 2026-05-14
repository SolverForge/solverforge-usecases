solverforge::planning_model! {
    root = "src/domain";

mod weekday;
pub use weekday::Weekday;

// @solverforge:begin domain-exports
mod timeslot;
mod teacher;
mod group;
mod lesson;
mod room;
mod plan;

pub use timeslot::Timeslot;
pub use teacher::Teacher;
pub use group::Group;
pub use lesson::Lesson;
pub use room::{Room, RoomKind};
pub use plan::Plan;
    // @solverforge:end domain-exports
}
