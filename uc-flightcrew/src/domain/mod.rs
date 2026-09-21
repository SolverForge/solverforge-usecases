solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
mod airport;
mod employee;
mod flight;
mod crew_assignment;
mod plan;

pub use airport::Airport;
pub use employee::Employee;
pub use flight::Flight;
pub use crew_assignment::CrewAssignment;
pub use plan::Plan;
    // @solverforge:end domain-exports
}
