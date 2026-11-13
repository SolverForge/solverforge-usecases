use super::*;

#[test]
fn demo_ids_round_trip() {
    for demo in available_demo_data() {
        assert_eq!(demo.id().parse::<DemoData>(), Ok(*demo));
    }
    assert_eq!(default_demo_data(), DemoData::Standard);
    assert!("SMALL".parse::<DemoData>().is_err());
}

#[test]
fn standard_plan_uses_every_place_and_starts_unrouted() {
    let plan = generate(DemoData::Standard);
    assert_eq!(plan.customers.len(), 48);
    assert_eq!(plan.chargers.len(), 14);
    assert_eq!(plan.vehicles.len(), 6);
    assert!(plan
        .vehicles
        .iter()
        .all(|vehicle| vehicle.visit_order.is_empty()));
}

#[test]
fn fleet_capacity_covers_total_demand() {
    for demo in available_demo_data() {
        let plan = generate(*demo);
        let demand: i32 = plan.customers.iter().map(|customer| customer.demand).sum();
        let capacity: i32 = plan.vehicles.iter().map(|vehicle| vehicle.capacity).sum();
        assert!(
            demand <= capacity,
            "{}: demand {demand} > capacity {capacity}",
            demo.id()
        );
    }
}

#[test]
fn ids_are_unique_and_coordinates_valid() {
    let plan = generate(DemoData::Standard);
    let mut ids: Vec<&str> = plan.customers.iter().map(|c| c.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), plan.customers.len());
    assert!(plan.customers.iter().all(|c| c.coord().is_ok()));
    assert!(plan.chargers.iter().all(|c| c.coord().is_ok()));
    assert!(plan
        .vehicles
        .iter()
        .all(|v| v.initial_charge_m <= v.battery_range_m));
}
