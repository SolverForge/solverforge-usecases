use std::cmp::Ordering;

use solverforge_maps::RoutingError;

use crate::domain::Plan;

use super::preparation::prepare_plan;
use super::scoring::evaluate_plan;
use super::types::DeliveryInsertionCandidate;

/// Ranks interactive insertion positions without running a full solve.
///
/// The selected delivery is removed from the current plan, the same routing
/// data used by solving is prepared, then every vehicle/position insertion is
/// preview-scored and sorted by feasibility first and travel quality second.
pub async fn rank_delivery_insertions(
    plan: &Plan,
    delivery_id: usize,
    limit: usize,
) -> Result<Vec<DeliveryInsertionCandidate>, RoutingError> {
    let mut base_plan = plan.clone();
    base_plan.normalize();
    base_plan.remove_delivery_assignments(delivery_id);
    prepare_plan(&mut base_plan).await?;

    let base_score = evaluate_plan(&base_plan);
    let mut candidates = Vec::new();

    for vehicle_idx in 0..base_plan.vehicles.len() {
        let vehicle_name = base_plan.vehicles[vehicle_idx].name.clone();
        for insert_index in 0..=base_plan.vehicles[vehicle_idx].delivery_order.len() {
            let mut candidate_plan = base_plan.clone();
            candidate_plan.vehicles[vehicle_idx]
                .delivery_order
                .insert(insert_index, delivery_id);
            let score = evaluate_plan(&candidate_plan);

            candidates.push(DeliveryInsertionCandidate {
                vehicle_id: candidate_plan.vehicles[vehicle_idx].id,
                vehicle_name: vehicle_name.clone(),
                insert_index,
                hard_score: score.hard_score(),
                soft_score: score.soft_score(),
                delta_hard: score.hard_score() - base_score.hard_score(),
                delta_soft: score.soft_score() - base_score.soft_score(),
                preview_plan: candidate_plan,
            });
        }
    }

    candidates.sort_by(compare_candidates);
    candidates.truncate(limit);
    Ok(candidates)
}

fn compare_candidates(
    left: &DeliveryInsertionCandidate,
    right: &DeliveryInsertionCandidate,
) -> Ordering {
    right
        .hard_score
        .cmp(&left.hard_score)
        .then_with(|| right.soft_score.cmp(&left.soft_score))
        .then_with(|| left.insert_index.cmp(&right.insert_index))
        .then_with(|| left.vehicle_id.cmp(&right.vehicle_id))
}
