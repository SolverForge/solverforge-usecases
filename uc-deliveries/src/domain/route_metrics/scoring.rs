use std::collections::HashSet;

use solverforge::prelude::HardSoftScore;
use crate::domain::{
    Delivery, DeliveryKind, DeliveryPreview, Plan, PlanPreview, VehiclePreview, VehiclePreviewStop,
};

use super::metrics::metrics_for_vehicle;
use super::types::{
    PlanScoreComponents, RouteStopMetrics, VehicleRouteMetrics, UNASSIGNED_DELIVERY_HARD_PENALTY,
    UNREACHABLE_HARD_PENALTY,
};

pub fn preview_for_plan(plan: &Plan) -> PlanPreview {
    let components = evaluate_plan(plan);
    let vehicle_metrics: Vec<VehicleRouteMetrics> = plan
        .vehicles
        .iter()
        .map(|vehicle| metrics_for_vehicle(plan, vehicle))
        .collect();

    let vehicles = plan
        .vehicles
        .iter()
        .zip(vehicle_metrics.iter())
        .map(|(vehicle, metrics)| {
            let stops = metrics
                .stops
                .iter()
                .map(|stop| vehicle_preview_stop(plan, stop))
                .collect();

            VehiclePreview {
                vehicle_id: vehicle.id,
                vehicle_name: vehicle.name.clone(),
                total_demand: metrics.total_demand,
                capacity_overage: metrics.capacity_overage,
                stop_count: metrics.stops.len(),
                total_travel_seconds: metrics.total_travel_seconds,
                total_wait_seconds: metrics.total_wait_seconds,
                total_service_seconds: metrics.total_service_seconds,
                total_late_seconds: metrics.total_late_seconds,
                start_time: metrics.start_time,
                end_time: metrics.end_time,
                stops,
            }
        })
        .collect();

    let mut deliveries = plan
        .deliveries
        .iter()
        .map(delivery_preview)
        .collect::<Vec<_>>();

    for (vehicle, metrics) in plan.vehicles.iter().zip(vehicle_metrics.iter()) {
        for stop in &metrics.stops {
            let preview = &mut deliveries[stop.delivery_id];
            preview.assigned_vehicle_id = Some(vehicle.id);
            preview.assigned_vehicle_name = Some(vehicle.name.clone());
            preview.sequence = Some(stop.sequence);
            preview.arrival_time = Some(stop.arrival_time);
            preview.service_start_time = Some(stop.service_start_time);
            preview.departure_time = Some(stop.departure_time);
            preview.late_seconds = Some(stop.late_seconds);
        }
    }

    PlanPreview {
        hard_score: components.hard_score(),
        soft_score: components.soft_score(),
        unassigned_delivery_ids: components.unassigned_delivery_ids(plan),
        vehicles,
        deliveries,
    }
}

/// Mirrors the constraint model for UI previews and insertion recommendations.
///
/// SolverForge constraints remain the authoritative score during solving. This
/// helper gives non-solver UI flows a cheap, explainable score breakdown from
/// the same route shadows and assignment coverage concepts.
pub fn evaluate_plan(plan: &Plan) -> PlanScoreComponents {
    let assigned: HashSet<usize> = plan
        .vehicles
        .iter()
        .flat_map(|vehicle| vehicle.delivery_order.iter().copied())
        .collect();

    let mut components = PlanScoreComponents {
        unassigned_count: plan
            .deliveries
            .iter()
            .filter(|delivery| !assigned.contains(&delivery.id))
            .count(),
        ..PlanScoreComponents::default()
    };

    for vehicle in &plan.vehicles {
        let metrics = metrics_for_vehicle(plan, vehicle);
        components.capacity_overage += i64::from(metrics.capacity_overage);
        components.late_seconds += metrics.total_late_seconds;
        components.unreachable_legs += metrics.unreachable_legs;
        components.travel_seconds += metrics.total_travel_seconds;
    }

    components
}

impl PlanScoreComponents {
    pub fn hard_score(&self) -> i64 {
        -((self.unassigned_count as i64 * UNASSIGNED_DELIVERY_HARD_PENALTY)
            + self.capacity_overage
            + self.late_seconds
            + (self.unreachable_legs as i64 * UNREACHABLE_HARD_PENALTY))
    }

    pub fn soft_score(&self) -> i64 {
        -self.travel_seconds
    }

    pub fn score(&self) -> HardSoftScore {
        HardSoftScore::of(self.hard_score(), self.soft_score())
    }

    pub fn unassigned_delivery_ids(&self, plan: &Plan) -> Vec<usize> {
        let assigned: HashSet<usize> = plan
            .vehicles
            .iter()
            .flat_map(|vehicle| vehicle.delivery_order.iter().copied())
            .collect();
        plan.deliveries
            .iter()
            .filter(|delivery| !assigned.contains(&delivery.id))
            .map(|delivery| delivery.id)
            .collect()
    }
}

fn vehicle_preview_stop(plan: &Plan, stop: &RouteStopMetrics) -> VehiclePreviewStop {
    let delivery = &plan.deliveries[stop.delivery_id];
    VehiclePreviewStop {
        delivery_id: delivery.id,
        label: delivery.label.clone(),
        kind: kind_name(delivery),
        sequence: stop.sequence,
        demand: delivery.demand,
        min_start_time: delivery.min_start_time,
        max_end_time: delivery.max_end_time,
        arrival_time: stop.arrival_time,
        service_start_time: stop.service_start_time,
        departure_time: stop.departure_time,
        travel_seconds_from_previous: stop.travel_seconds_from_previous,
        wait_seconds: stop.wait_seconds,
        late_seconds: stop.late_seconds,
    }
}

fn delivery_preview(delivery: &Delivery) -> DeliveryPreview {
    DeliveryPreview {
        delivery_id: delivery.id,
        label: delivery.label.clone(),
        kind: kind_name(delivery),
        demand: delivery.demand,
        min_start_time: delivery.min_start_time,
        max_end_time: delivery.max_end_time,
        service_duration: delivery.service_duration,
        assigned_vehicle_id: None,
        assigned_vehicle_name: None,
        sequence: None,
        arrival_time: None,
        service_start_time: None,
        departure_time: None,
        late_seconds: None,
    }
}

fn kind_name(delivery: &Delivery) -> String {
    match delivery.kind {
        DeliveryKind::Residential => "residential",
        DeliveryKind::Business => "business",
        DeliveryKind::Restaurant => "restaurant",
        DeliveryKind::Other => "other",
    }
    .to_string()
}
