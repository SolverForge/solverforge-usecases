use super::*;
use crate::data::data_seed::types::LocationData;
use solverforge_maps::{BoundingBox, Coord, NetworkConfig, RoadNetwork};

#[test]
fn parses_demo_data_ids_case_insensitively() {
    assert!(matches!(
        "philadelphia".parse::<DemoData>(),
        Ok(DemoData::Philadelphia)
    ));
    assert!(matches!(
        "HARTFORD".parse::<DemoData>(),
        Ok(DemoData::Hartford)
    ));
}

#[test]
fn generates_city_demo_plan() {
    for (demo, expected_stops, expected_name) in [
        (
            DemoData::Philadelphia,
            visit_count(philadelphia::VISIT_GROUPS),
            "Philadelphia",
        ),
        (
            DemoData::Hartford,
            visit_count(hartford::VISIT_GROUPS),
            "Hartford",
        ),
        (
            DemoData::Firenze,
            visit_count(firenze::VISIT_GROUPS),
            "Firenze",
        ),
    ] {
        let plan = generate(demo);
        assert_eq!(plan.name, expected_name);
        assert_eq!(plan.vehicles.len(), 10);
        assert_eq!(plan.deliveries.len(), expected_stops);
        assert!(plan
            .vehicles
            .iter()
            .all(|vehicle| vehicle.delivery_order.is_empty()));
    }
}

fn visit_count(groups: &[&[LocationData]]) -> usize {
    groups.iter().map(|group| group.len()).sum()
}

#[test]
fn demo_delivery_counts_scale_with_ten_vehicles() {
    assert_eq!(generate(DemoData::Philadelphia).deliveries.len(), 82);
    assert_eq!(generate(DemoData::Hartford).deliveries.len(), 50);
    assert_eq!(generate(DemoData::Firenze).deliveries.len(), 80);
}

#[test]
fn demo_plans_have_enough_vehicle_capacity() {
    for demo in [
        DemoData::Philadelphia,
        DemoData::Hartford,
        DemoData::Firenze,
    ] {
        let plan = generate(demo);
        let total_capacity: i32 = plan.vehicles.iter().map(|vehicle| vehicle.capacity).sum();
        let total_demand: i32 = plan.deliveries.iter().map(|delivery| delivery.demand).sum();

        assert!(
            total_capacity >= total_demand,
            "{demo:?} demo should be capacity-feasible before route ordering: capacity={total_capacity}, demand={total_demand}"
        );
    }
}

#[tokio::test]
async fn live_demo_locations_are_mutually_reachable_when_enabled() {
    if std::env::var("SOLVERFORGE_RUN_LIVE_TESTS").ok().as_deref() != Some("1") {
        return;
    }

    for demo in [
        DemoData::Philadelphia,
        DemoData::Hartford,
        DemoData::Firenze,
    ] {
        let plan = generate(demo);
        let mut named_coords = Vec::new();
        for delivery in &plan.deliveries {
            named_coords.push((
                format!("delivery {}", delivery.label),
                delivery.coord().unwrap(),
            ));
        }
        for vehicle in &plan.vehicles {
            named_coords.push((
                format!("depot {}", vehicle.name),
                vehicle.depot_coord().unwrap(),
            ));
        }

        let coords: Vec<Coord> = named_coords.iter().map(|(_, coord)| *coord).collect();
        let bbox = BoundingBox::from_coords(&coords).expand_for_routing(&coords);
        let network = RoadNetwork::load_or_fetch(&bbox, &NetworkConfig::default(), None)
            .await
            .unwrap();
        let matrix = network.compute_matrix(&coords, None).await;
        let unreachable = matrix
            .unreachable_pairs()
            .into_iter()
            .map(|(from_idx, to_idx)| {
                let from_name = &named_coords[from_idx].0;
                let to_name = &named_coords[to_idx].0;
                format!("{from_name} -> {to_name}")
            })
            .collect::<Vec<_>>();

        assert!(
            unreachable.is_empty(),
            "{demo:?} has unreachable directed routes: {unreachable:?}"
        );
    }
}
