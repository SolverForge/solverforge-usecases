use chrono::Timelike;
use solverforge::{ConstraintSet, SolverEvent, SolverManager};
use std::collections::BTreeMap;

use super::{generate, DemoData};
use crate::domain::Plan;

// Slow end-to-end acceptance test for the published benchmark instance.

fn schedule() -> Plan {
    generate(DemoData::Large)
}

#[test]
#[ignore = "slow acceptance test for the canonical quickstart dataset"]
fn large_demo_solves_to_feasible_terminal_state() {
    static MANAGER: SolverManager<Plan> = SolverManager::new();

    let schedule = schedule();
    let (job_id, mut receiver) = MANAGER.solve(schedule).expect("job should start");
    let mut completed_score = None;

    while let Some(event) = receiver.blocking_recv() {
        match event {
            SolverEvent::Completed { solution, .. } => {
                completed_score = solution.score;
                if let Some(score) = solution.score {
                    if score.hard_score() != solverforge::HardSoftDecimalScore::ZERO {
                        let mut mismatches = BTreeMap::<(String, u32, String), usize>::new();
                        for shift in &solution.shifts {
                            let Some(employee_idx) = shift.employee_idx else {
                                continue;
                            };
                            let employee = &solution.employees[employee_idx];
                            if !employee.skills.contains(&shift.required_skill) {
                                *mismatches
                                    .entry((
                                        shift.location.clone(),
                                        shift.start.time().hour(),
                                        shift.required_skill.clone(),
                                    ))
                                    .or_default() += 1;
                            }
                        }
                        eprintln!("large demo skill mismatches: {mismatches:?}");

                        let constraints = crate::constraints::create_constraints();
                        let analyses = constraints.evaluate_detailed(&solution);
                        let hard_breakdown: Vec<_> = analyses
                            .into_iter()
                            .filter(|analysis| {
                                analysis.score.hard_score()
                                    != solverforge::HardSoftDecimalScore::ZERO
                            })
                            .map(|analysis| {
                                format!("{}={}", analysis.constraint_ref.name, analysis.score)
                            })
                            .collect();
                        eprintln!("large demo hard breakdown: {}", hard_breakdown.join(", "));
                    }
                }
                break;
            }
            SolverEvent::Failed { error, .. } => {
                panic!("large demo solve failed unexpectedly: {error}");
            }
            _ => {}
        }
    }

    let score = completed_score.expect("expected a completed score");
    assert_eq!(score.hard_score(), solverforge::HardSoftDecimalScore::ZERO);

    MANAGER.delete(job_id).expect("delete completed job");
}
