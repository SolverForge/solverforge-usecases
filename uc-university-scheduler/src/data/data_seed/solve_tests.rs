use solverforge::{ConstraintSet, SolverEvent, SolverManager};
use std::collections::BTreeSet;

use super::{generate, DemoData};
use crate::domain::Plan;

// Static manager — must be 'static for retained job execution.
static MANAGER: SolverManager<Plan> = SolverManager::new();

fn schedule() -> Plan {
    generate(DemoData::Large)
}

/// Slow end-to-end acceptance test for the large dataset.
///
/// This verifies that the solver starts from an unassigned schedule, reaches a
/// hard/medium-feasible timetable, and keeps a visible soft optimization score.
#[test]
#[ignore = "slow acceptance test for the large dataset"]
fn large_demo_solves_to_feasible_progressing_schedule() {
    let plan = schedule();
    let initial_score = crate::constraints::create_constraints().evaluate_all(&plan);
    assert_eq!(
        initial_score,
        solverforge::HardMediumSoftScore::of_medium(-600),
        "The generated demo must start unassigned, not already solved"
    );

    let (job_id, mut receiver) = MANAGER.solve(plan).expect("job should start");
    let mut completed_score = None;
    let mut completed_solution = None;
    let mut observed_scores = Vec::new();

    while let Some(event) = receiver.blocking_recv() {
        if let Some(score) = event.metadata().current_score {
            observed_scores.push(score);
        }

        match event {
            SolverEvent::Completed { solution, .. } => {
                completed_score = solution.score;
                completed_solution = Some(solution);
                break;
            }
            SolverEvent::Failed { error, .. } => {
                panic!("demo solve failed unexpectedly: {error}");
            }
            _ => {}
        }
    }

    let score = completed_score.expect("expected a completed score");
    let solution = completed_solution.expect("expected a completed solution");

    // The best solution must satisfy both the hard feasibility rules and the
    // medium-level assignment requirements while retaining soft optimization
    // pressure that lets the UI show continued score movement.
    assert_eq!(
        score.hard(),
        0,
        "Expected hard-feasible solution, but got: {}",
        score
    );
    assert_eq!(
        score.medium(),
        0,
        "Expected all lessons assigned, but got: {}",
        score
    );
    assert!(
        score.soft() < 0,
        "Expected remaining soft penalties for realistic timetable quality, got: {}",
        score
    );
    assert!(
        score.medium() > initial_score.medium(),
        "Expected terminal score to improve from the unassigned medium penalty"
    );
    assert!(
        observed_scores.iter().any(|score| *score == initial_score),
        "Expected event stream to expose the unassigned initial score"
    );
    assert!(
        observed_scores
            .iter()
            .any(|score| score.hard() == 0 && score.medium() == 0 && score.soft() < 0),
        "Expected event stream to expose a feasible soft-scored timetable"
    );

    let lesson_count = solution.lessons.len();
    let assigned_timeslots = solution
        .lessons
        .iter()
        .filter(|l| l.timeslot_idx.is_some())
        .count();
    let assigned_rooms = solution
        .lessons
        .iter()
        .filter(|l| l.room_idx.is_some())
        .count();

    assert_eq!(
        assigned_timeslots, lesson_count,
        "Every lesson must have a timeslot assignment"
    );
    assert_eq!(
        assigned_rooms, lesson_count,
        "Every lesson must have a room assignment"
    );

    let distinct_timeslots = solution
        .lessons
        .iter()
        .filter_map(|lesson| lesson.timeslot_idx)
        .collect::<BTreeSet<_>>()
        .len();
    let distinct_rooms = solution
        .lessons
        .iter()
        .filter_map(|lesson| lesson.room_idx)
        .collect::<BTreeSet<_>>()
        .len();

    assert!(
        distinct_timeslots > 1,
        "The solved timetable must not collapse every lesson into one timeslot"
    );
    assert!(
        distinct_rooms > 1,
        "The solved timetable must not collapse every lesson into one room"
    );

    let constraints = crate::constraints::create_constraints();
    let hard_or_medium_constraints: Vec<_> = constraints
        .evaluate_detailed(&solution)
        .into_iter()
        .filter(|analysis| analysis.score.hard() != 0 || analysis.score.medium() != 0)
        .map(|analysis| format!("{}={}", analysis.constraint_ref.name, analysis.score))
        .collect();
    assert!(
        hard_or_medium_constraints.is_empty(),
        "Expected all hard/medium constraints to score zero, got: {}",
        hard_or_medium_constraints.join(", ")
    );

    eprintln!(
        "Solution: {} lessons, {} timeslots assigned, {} rooms assigned, score {}",
        lesson_count, assigned_timeslots, assigned_rooms, score
    );

    MANAGER.delete(job_id).expect("delete completed job");
}
