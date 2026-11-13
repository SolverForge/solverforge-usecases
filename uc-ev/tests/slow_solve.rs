//! End-to-end acceptance solve over the real Bergamo road network.
//!
//! Ignored by default: it needs the OpenStreetMap road data (downloaded once
//! into `.osm_cache/`) and runs the full solver budget. `make test-slow` runs
//! it in the release profile.

use solverforge::{SolverEvent, SolverManager};
use solverforge_ev::data::{generate, DemoData};
use solverforge_ev::domain::{prepare_plan, Plan, RouteEventKind};

static MANAGER: SolverManager<Plan> = SolverManager::new();

#[test]
#[ignore = "slow end-to-end acceptance solve on the live road network"]
fn standard_demo_solves_to_zero_hard_with_every_customer_routed_once() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let mut plan = generate(DemoData::Standard);
    runtime
        .block_on(prepare_plan(&mut plan))
        .expect("road network should load");

    let (job_id, mut receiver) = MANAGER.solve(plan).expect("solve should start");
    let solution = loop {
        match receiver.blocking_recv().expect("terminal solver event") {
            SolverEvent::Completed { solution, .. } => break solution,
            SolverEvent::Failed { error, .. } => panic!("solve failed: {error}"),
            _ => {}
        }
    };
    let score = solution.score.expect("terminal score");
    println!("STANDARD final score: {score}");
    assert_eq!(score.hard(), 0, "expected a feasible plan: {score}");

    let mut routed: Vec<usize> = solution
        .vehicles
        .iter()
        .flat_map(|vehicle| vehicle.visit_order.iter().copied())
        .collect();
    routed.sort_unstable();
    assert_eq!(routed, (0..solution.customers.len()).collect::<Vec<_>>());

    for vehicle in &solution.vehicles {
        let charges = vehicle
            .schedule
            .iter()
            .filter(|event| event.kind == RouteEventKind::Charge)
            .count();
        println!(
            "{}: {} stops, {} km, {} charge(s), battery shortfall {} m",
            vehicle.name,
            vehicle.visit_order.len(),
            vehicle.route_distance_m / 1_000,
            charges,
            vehicle.route_shortfall_m
        );
        assert!(
            vehicle
                .schedule
                .iter()
                .all(|event| event.charge_on_arrival_m >= 0),
            "{} runs out of battery",
            vehicle.name
        );
    }
    MANAGER.delete(job_id).expect("delete completed job");
}
