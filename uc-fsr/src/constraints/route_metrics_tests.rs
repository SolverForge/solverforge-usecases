use crate::domain::{
    route_metrics::{leg_for, route_stats},
    FieldServicePlan, Location, ServiceVisit, ServiceVisitInit, TechnicianRoute,
    TechnicianRouteInit, TravelLeg, TravelLegInit,
};
use solverforge::ConstraintSet;

#[test]
fn route_stats_accounts_for_travel_service_and_lateness() {
    let plan = sample_plan(vec![0, 1]);
    let stats = route_stats(&plan, &plan.technician_routes[0]);

    assert_eq!(stats.travel_seconds, 1_800);
    assert_eq!(stats.service_minutes, 75);
    assert_eq!(stats.late_minutes, 0);
    assert_eq!(stats.route_minutes, 125);
    assert_eq!(stats.overtime_minutes, 55);
    assert_eq!(stats.valid_visits, 2);
    assert_eq!(stats.scored_travel_legs, 3);
    assert_eq!(stats.missing_skill_visits, 0);
    assert_eq!(stats.missing_part_visits, 1);
}

#[test]
fn travel_leg_lookup_prefers_row_major_contract() {
    let plan = sample_plan(vec![0]);
    let leg = leg_for(&plan, 0, 1).expect("leg should exist");

    assert_eq!(leg.id, "leg-0-1");
    assert!(leg.reachable);
}

#[test]
fn field_service_plan_deserialize_restores_transient_indexes_and_shadows() {
    let plan = sample_plan(vec![1, 0]);
    let decoded: FieldServicePlan =
        serde_json::from_value(serde_json::to_value(&plan).expect("plan should serialize"))
            .expect("plan should deserialize");

    let indexes = decoded
        .service_visits
        .iter()
        .map(|visit| visit.index)
        .collect::<Vec<_>>();
    assert_eq!(indexes, vec![0, 1]);
    assert_eq!(decoded.technician_routes[0].route_valid_visits, 2);
    assert!(decoded.technician_routes[0].route_travel_seconds > 0);
}

#[test]
fn full_constraint_set_reports_expected_hard_penalties() {
    let constraints = crate::constraints::create_constraints();
    let score = constraints.evaluate_all(&sample_plan(vec![0, 1]));

    assert_eq!(score.hard(), -56);
    assert!(score.soft() < 0);
}

#[test]
fn stock_route_constraints_report_matching_routes() {
    let constraints = crate::constraints::create_constraints();
    let results = constraints.evaluate_each(&sample_plan(vec![0, 1]));
    let match_count = |name: &str| {
        results
            .iter()
            .find(|result| result.name == name)
            .map(|result| result.match_count)
            .unwrap_or_else(|| panic!("missing constraint result for {name}"))
    };

    assert_eq!(match_count("Balance Workload"), 1);
    assert_eq!(match_count("Minimize Travel"), 1);
    assert_eq!(match_count("Priority Slack"), 1);
    assert_eq!(match_count("Required Parts"), 1);
    assert_eq!(match_count("Shift Capacity"), 1);
    assert_eq!(match_count("Territory Affinity"), 1);
}

fn sample_plan(visits: Vec<usize>) -> FieldServicePlan {
    let locations = vec![
        Location::new(
            "loc-0",
            "Hub",
            "Hub".to_string(),
            45_700_000,
            9_670_000,
            "depot".to_string(),
        ),
        Location::new(
            "loc-1",
            "Customer 1",
            "Customer 1".to_string(),
            45_710_000,
            9_680_000,
            "customer".to_string(),
        ),
        Location::new(
            "loc-2",
            "Customer 2",
            "Customer 2".to_string(),
            45_720_000,
            9_690_000,
            "customer".to_string(),
        ),
    ];
    let service_visits = vec![
        ServiceVisit::new(ServiceVisitInit {
            id: "visit-0".to_string(),
            name: "Boiler".to_string(),
            customer: "Customer 1".to_string(),
            location_idx: 1,
            duration_minutes: 30,
            earliest_minute: 510,
            latest_minute: 540,
            required_skill_mask: 0b001,
            required_parts_mask: 0b010,
            priority: 3,
            territory: "center".to_string(),
        }),
        ServiceVisit::new(ServiceVisitInit {
            id: "visit-1".to_string(),
            name: "Lift".to_string(),
            customer: "Customer 2".to_string(),
            location_idx: 2,
            duration_minutes: 45,
            earliest_minute: 540,
            latest_minute: 570,
            required_skill_mask: 0b001,
            required_parts_mask: 0b100,
            priority: 2,
            territory: "center".to_string(),
        }),
    ];
    let travel_legs = row_major_legs(3);
    let mut route = TechnicianRoute::new(TechnicianRouteInit {
        id: "route-0".to_string(),
        technician_id: "tech-0".to_string(),
        technician_name: "Ada".to_string(),
        color: "#2563eb".to_string(),
        start_location_idx: 0,
        end_location_idx: 0,
        shift_start_minute: 480,
        shift_end_minute: 585,
        max_route_minutes: 90,
        skill_mask: 0b001,
        inventory_mask: 0b010,
        territory: "center".to_string(),
    });
    route.visits = visits;

    FieldServicePlan::new(locations, service_visits, travel_legs, vec![route])
}

fn row_major_legs(width: usize) -> Vec<TravelLeg> {
    (0..width)
        .flat_map(|from| {
            (0..width).map(move |to| {
                let same = from == to;
                TravelLeg::new(TravelLegInit {
                    id: format!("leg-{from}-{to}"),
                    name: format!("leg-{from}-{to}"),
                    from_location_idx: from,
                    to_location_idx: to,
                    duration_seconds: if same { 0 } else { 600 },
                    distance_meters: if same { 0 } else { 2_000 },
                    reachable: true,
                })
            })
        })
        .collect()
}
