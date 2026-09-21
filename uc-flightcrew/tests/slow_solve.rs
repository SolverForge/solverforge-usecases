use solverforge::{SolverEvent, SolverManager};
use solverforge_flightcrew::{
    data::{generate, DemoData},
    domain::Plan,
};

static MANAGER: SolverManager<Plan> = SolverManager::new();

#[test]
#[ignore = "slow end-to-end acceptance solve"]
fn standard_demo_solves_to_feasible_terminal_state() {
    let (job_id, mut receiver) = MANAGER
        .solve(generate(DemoData::Standard))
        .expect("solve should start");
    let solution = loop {
        match receiver.blocking_recv().expect("terminal solver event") {
            SolverEvent::Completed { solution, .. } => break solution,
            SolverEvent::Failed { error, .. } => panic!("solve failed: {error}"),
            _ => {}
        }
    };
    let score = solution.score.expect("terminal score");
    assert_eq!(
        score.hard(),
        0,
        "expected hard-feasible crew schedule: {score}"
    );
    assert!(solution
        .crew_assignments
        .iter()
        .all(|assignment| assignment.employee_idx.is_some()));
    MANAGER.delete(job_id).expect("delete completed job");
}
