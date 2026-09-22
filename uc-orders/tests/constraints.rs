use solverforge::{ConstraintSet, HardSoftScore};
use solverforge_orders::{
    constraints::create_constraints,
    domain::{PickStep, Plan, Side, Trolley, WarehouseLocation},
};

fn location(shelf: &str, side: Side, row: i64) -> WarehouseLocation {
    WarehouseLocation::new(shelf, side, row)
}

fn step(index: usize, order: &str, volume: i64, location: WarehouseLocation) -> PickStep {
    PickStep::new(
        format!("step-{index}"),
        format!("Product {index}"),
        format!("product-{index}"),
        order,
        format!("item-{index}"),
        volume,
        location,
    )
}

fn scored_plan(steps: Vec<PickStep>, bucket_count: i64, route: Vec<usize>) -> Plan {
    let mut trolley = Trolley::new("1", bucket_count, 48_000, location("(A,1)", Side::Left, 5));
    trolley.step_order = route;
    let mut plan = Plan::new(steps, vec![trolley]);
    plan.refresh_trolley_route_shadows(0);
    plan
}

fn score(plan: &Plan, name: &str) -> HardSoftScore {
    create_constraints()
        .evaluate_each(plan)
        .into_iter()
        .find(|result| result.name == name)
        .expect("constraint result")
        .score
}

#[test]
fn required_buckets_sums_ceiling_per_order_and_penalizes_each_excess_bucket() {
    let plan = scored_plan(
        vec![
            step(0, "A", 40_000, location("(B,1)", Side::Left, 3)),
            step(1, "A", 40_000, location("(A,1)", Side::Left, 8)),
            step(2, "B", 48_001, location("(C,1)", Side::Left, 4)),
        ],
        2,
        vec![0, 1, 2],
    );

    assert_eq!(plan.trolleys[0].required_bucket_count, 4);
    assert_eq!(score(&plan, "required_buckets"), HardSoftScore::of(-2, 0));
}

#[test]
fn order_incidence_preserves_one_thousand_point_baseline_per_distinct_order() {
    let plan = scored_plan(
        vec![
            step(0, "A", 1, location("(A,1)", Side::Left, 6)),
            step(1, "A", 1, location("(A,1)", Side::Left, 7)),
            step(2, "B", 1, location("(A,1)", Side::Left, 8)),
        ],
        4,
        vec![0, 1, 2],
    );

    assert_eq!(
        score(&plan, "order_incidence"),
        HardSoftScore::of(0, -2_000)
    );
}

#[test]
fn route_distance_scores_the_closed_route_in_meters() {
    let plan = scored_plan(
        vec![step(0, "A", 1, location("(A,1)", Side::Left, 8))],
        4,
        vec![0],
    );

    assert_eq!(plan.trolleys[0].route_distance_meters, 6);
    assert_eq!(score(&plan, "route_distance"), HardSoftScore::of(0, -6));
}
