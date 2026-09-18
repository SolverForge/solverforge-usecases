use std::time::Duration;

use solverforge::SolverLifecycleState;
use solverforge_fleet::data::{generate, DemoData};
use solverforge_fleet::solver::SolverService;

#[tokio::test]
#[ignore = "slow acceptance solve"]
async fn baseline_full_solves_to_hard_feasible_terminal_state() {
    let service = SolverService::new();
    let job_id = service
        .start_job(generate(DemoData::Large))
        .expect("start BASELINE_FULL solve");

    let mut terminal = None;
    for _ in 0..750 {
        let status = service.get_status(&job_id).expect("read job status");
        if matches!(
            status.lifecycle_state,
            SolverLifecycleState::Completed
                | SolverLifecycleState::Cancelled
                | SolverLifecycleState::Failed
        ) {
            terminal = Some(status.lifecycle_state);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert_eq!(terminal, Some(SolverLifecycleState::Completed));
    let snapshot = service
        .get_snapshot(&job_id, None)
        .expect("completed job has a snapshot");
    let score = snapshot.solution.score.expect("completed plan has a score");
    assert_eq!(score.hard(), 0, "terminal plan must be hard feasible");
    service.delete(&job_id).expect("delete terminal job");
}
