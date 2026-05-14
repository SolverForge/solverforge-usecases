//! Planning-model manifest and domain-layer exports.
//!
//! `planning_model!` is the SolverForge boundary for this app. Keep the exports
//! in the same conceptual order as `solverforge.app.toml`: facts, planning
//! entity, then solution.

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
