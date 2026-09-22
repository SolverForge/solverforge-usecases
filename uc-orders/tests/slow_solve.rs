use solverforge::{SolverEvent, SolverManager};
use solverforge_orders::{
    data::{generate, DemoData},
    domain::Plan,
};

static MANAGER: SolverManager<Plan> = SolverManager::new();

#[test]
#[ignore = "slow end-to-end acceptance solve"]
fn standard_demo_solves_to_zero_hard_with_every_step_assigned_once() {
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
    assert_eq!(score.hard(), 0, "expected a feasible picking plan: {score}");

    let mut assigned = solution
        .trolleys
        .iter()
        .flat_map(|trolley| trolley.step_order.iter().copied())
        .collect::<Vec<_>>();
    assigned.sort_unstable();
    assert_eq!(assigned, (0..solution.pick_steps.len()).collect::<Vec<_>>());
    MANAGER.delete(job_id).expect("delete completed job");
}
