use super::*;
use crate::data::{generate, DemoData};
use crate::domain::{prepare_plan, DeliveryKind, UNASSIGNED_DELIVERY_HARD_PENALTY};
use solverforge::{Director, ScoreDirector, SolverConfig, SolverEvent, SolverManager};

fn tiny_plan() -> Plan {
    Plan::new(
        "tiny",
        vec![
            Delivery::new(
                0,
                "A",
                DeliveryKind::Residential,
                (39.9526, -75.1652),
                1,
                (8 * 3600, 18 * 3600),
                10 * 60,
            ),
            Delivery::new(
                1,
                "B",
                DeliveryKind::Business,
                (39.9626, -75.1752),
                1,
                (8 * 3600, 18 * 3600),
                10 * 60,
            ),
        ],
        vec![Vehicle::new(0, "Van 1", 4, 39.9526, -75.1652, 8 * 3600)],
    )
}

fn prepared_tiny_plan_with_route() -> Plan {
    let mut plan = tiny_plan();
    plan.routing_mode = RoutingMode::StraightLine;
    plan.vehicles[0].delivery_order = vec![0, 1];
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(async {
            prepare_plan(&mut plan)
                .await
                .expect("plan preparation should work");
        });
    plan
}

#[test]
fn score_director_populates_vehicle_route_shadows() {
    let plan = prepared_tiny_plan_with_route();
    assert_eq!(
        plan.vehicles[0].route_total_demand, 0,
        "prepared transport data should not eagerly populate solver shadows"
    );

    let mut director = ScoreDirector::with_descriptor(
        plan,
        crate::constraints::create_constraints(),
        Plan::descriptor(),
        Plan::entity_count,
    );
    let score = director.calculate_score();
    let vehicle = &director.working_solution().vehicles[0];

    assert_eq!(vehicle.total_assigned_demand(), 2);
    assert_eq!(vehicle.capacity_overage(), 0);
    assert!(
        vehicle.total_travel_seconds() > 0,
        "route travel should be maintained as a shadow value"
    );
    assert_eq!(score.hard(), 0);
}

#[test]
fn vehicle_route_shadows_refresh_after_list_variable_changes() {
    let plan = prepared_tiny_plan_with_route();
    let mut director = ScoreDirector::with_descriptor(
        plan,
        crate::constraints::create_constraints(),
        Plan::descriptor(),
        Plan::entity_count,
    );
    director.calculate_score();
    assert_eq!(
        director.working_solution().vehicles[0].total_assigned_demand(),
        2
    );

    director.before_variable_changed(0, 0);
    director.working_solution_mut().vehicles[0]
        .delivery_order
        .clear();
    director.after_variable_changed(0, 0);
    let score = director.calculate_score();

    let vehicle = &director.working_solution().vehicles[0];
    assert_eq!(vehicle.total_assigned_demand(), 0);
    assert_eq!(vehicle.total_travel_seconds(), 0);
    assert_eq!(vehicle.time_window_violation_seconds(), 0);
    assert_eq!(score.hard(), -(2 * UNASSIGNED_DELIVERY_HARD_PENALTY));
}

#[test]
fn generated_list_runtime_is_non_trivial_and_builds_routes() {
    static MANAGER: SolverManager<Plan> = SolverManager::new();

    let mut plan = tiny_plan();
    plan.routing_mode = RoutingMode::StraightLine;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(async {
            prepare_plan(&mut plan)
                .await
                .expect("plan preparation should work");
        });

    assert!(
        Plan::test_has_list_variable(),
        "delivery plan should expose a list variable"
    );
    assert_eq!(Plan::test_total_list_entities(&plan), 1);
    assert_eq!(Plan::test_total_list_elements(&plan), 2);
    assert!(
        !Plan::test_is_trivial(&plan),
        "prepared plan should not be trivial"
    );

    let config =
        SolverConfig::from_toml_str(include_str!("../../solver.toml")).expect("valid config");
    assert_eq!(
        Plan::test_phase_count(&config),
        3,
        "expected Clarke-Wright construction + list k-opt + local search"
    );

    let (job_id, mut receiver) = MANAGER.solve(plan).expect("solve should start");
    let mut saw_non_empty_best = false;
    loop {
        match receiver
            .blocking_recv()
            .expect("event stream should reach a terminal event")
        {
            SolverEvent::BestSolution { solution, .. } => {
                if solution
                    .vehicles
                    .iter()
                    .any(|vehicle| !vehicle.delivery_order.is_empty())
                {
                    saw_non_empty_best = true;
                    MANAGER.cancel(job_id).expect("job cancel should succeed");
                }
            }
            SolverEvent::Completed { .. } | SolverEvent::Cancelled { .. } => break,
            SolverEvent::Failed { error, .. } => {
                panic!("solve unexpectedly failed: {error}");
            }
            SolverEvent::Progress { .. }
            | SolverEvent::PauseRequested { .. }
            | SolverEvent::Paused { .. }
            | SolverEvent::Resumed { .. } => {}
        }
    }
    MANAGER
        .delete(job_id)
        .expect("completed test job should delete");

    assert!(
        saw_non_empty_best,
        "expected a non-empty best solution before cancellation"
    );
}

#[test]
fn seeded_philadelphia_plan_emits_a_non_empty_best_solution() {
    static MANAGER: SolverManager<Plan> = SolverManager::new();

    let mut plan = generate(DemoData::Philadelphia);
    plan.routing_mode = RoutingMode::StraightLine;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(async {
            prepare_plan(&mut plan)
                .await
                .expect("plan preparation should work");
        });

    let (job_id, mut receiver) = MANAGER.solve(plan).expect("solve should start");
    let mut saw_non_empty_best = false;
    let mut first_non_empty_best: Option<Plan> = None;
    loop {
        match receiver
            .blocking_recv()
            .expect("event stream should reach a terminal event")
        {
            SolverEvent::BestSolution { solution, .. } => {
                if solution
                    .vehicles
                    .iter()
                    .any(|vehicle| !vehicle.delivery_order.is_empty())
                {
                    saw_non_empty_best = true;
                    first_non_empty_best.get_or_insert(solution.clone());
                    MANAGER.cancel(job_id).expect("job cancel should succeed");
                }
            }
            SolverEvent::Completed { .. } | SolverEvent::Cancelled { .. } => break,
            SolverEvent::Failed { error, .. } => {
                panic!("solve unexpectedly failed: {error}");
            }
            SolverEvent::Progress { .. }
            | SolverEvent::PauseRequested { .. }
            | SolverEvent::Paused { .. }
            | SolverEvent::Resumed { .. } => {}
        }
    }
    MANAGER
        .delete(job_id)
        .expect("completed test job should delete");

    assert!(
        saw_non_empty_best,
        "expected a non-empty best solution for the seeded Philadelphia plan"
    );
    let best = first_non_empty_best.expect("should retain the first non-empty best solution");
    assert!(
        best.vehicles
            .iter()
            .any(|vehicle| vehicle.delivery_order.len() > 1),
        "expected at least one multi-stop route after construction"
    );
    let director = ScoreDirector::with_descriptor(
        best.clone(),
        crate::constraints::create_constraints(),
        Plan::descriptor(),
        Plan::entity_count,
    );
    assert_eq!(director.entity_count(0), Some(best.vehicles.len()));
}
