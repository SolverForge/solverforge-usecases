use solverforge::ConstraintSet;
use solverforge_flightcrew::{
    constraints::create_constraints,
    data::{generate, DemoData},
};

fn score(plan: &solverforge_flightcrew::domain::Plan, name: &str) -> solverforge::HardSoftScore {
    create_constraints()
        .evaluate_each(plan)
        .into_iter()
        .find(|result| result.name == name)
        .expect("constraint result")
        .score
}

#[test]
fn every_open_seat_is_penalized() {
    let plan = generate(DemoData::Standard);
    assert_eq!(score(&plan, "assigned_crew").hard(), -420_000);
}

#[test]
fn skill_rule_scores_each_mismatched_seat() {
    let mut plan = generate(DemoData::Standard);
    for assignment in &mut plan.crew_assignments {
        assignment.employee_idx = Some(0);
    }
    assert_eq!(score(&plan, "required_skill").hard(), -2_200);
}

#[test]
fn overlapping_seats_cannot_share_one_employee() {
    let mut plan = generate(DemoData::Standard);
    plan.crew_assignments[0].employee_idx = Some(0);
    plan.crew_assignments[1].employee_idx = Some(0);
    assert_eq!(score(&plan, "flight_conflict").hard(), -10);
}
