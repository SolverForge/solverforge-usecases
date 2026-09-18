use std::collections::BTreeMap;

use super::furnace_seed::slot_preserves_shift_ownership;
use super::*;
use crate::constraints::create_constraints;
use crate::domain::{
    calculate_changeover_cost_for_sequence, furnace_accepts_work_order,
    furnace_assignment_construction_entity_order_key,
    furnace_assignment_construction_value_order_key, operator_shift_construction_entity_order_key,
    operator_shift_construction_value_order_key, HeatTreatmentProcess::*, PriorityBand,
    HORIZON_MINUTES, NUM_TIME_SLOTS, TIME_STEP,
};
use solverforge::ConstraintSet;

struct ScheduledInterval {
    assignment_idx: usize,
    start: usize,
    end: usize,
}

fn plan_for(demo: DemoData) -> crate::domain::Plan {
    build_demo_plan_for(demo)
}

#[test]
fn default_demo_fleet_matches_prd_mix() {
    let plan = build_demo_plan();
    assert_eq!(plan.furnaces.len(), 11);

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for furnace in &plan.furnaces {
        *counts.entry(furnace.furnace_type.to_string()).or_default() += 1;
    }

    assert_eq!(counts.get("Chamber"), Some(&5));
    assert_eq!(counts.get("Pit"), Some(&4));
    assert_eq!(counts.get("Car Hearth"), Some(&1));
    assert_eq!(counts.get("Aluminum"), Some(&1));

    let carburizing_pits = plan
        .furnaces
        .iter()
        .filter(|furnace| furnace.supported_processes.contains(&Carburizing))
        .count();
    let nitriding_pits = plan
        .furnaces
        .iter()
        .filter(|furnace| furnace.supported_processes == [Nitriding])
        .count();

    assert_eq!(carburizing_pits, 2);
    assert_eq!(nitriding_pits, 2);
}

#[test]
fn default_demo_workforce_matches_prd_roles() {
    let plan = build_demo_plan();
    assert_eq!(plan.operators.len(), 39);

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for operator in &plan.operators {
        *counts.entry(operator.role.to_string()).or_default() += 1;
    }

    assert_eq!(counts.get("Shift Lead"), Some(&13));
    assert_eq!(counts.get("Furnace Operator"), Some(&15));
    assert_eq!(counts.get("Maintenance Technician"), Some(&8));
    assert_eq!(counts.get("Material Handler"), Some(&2));
    assert_eq!(counts.get("Quality Control"), Some(&1));
}

#[test]
fn demo_keeps_operator_ids_index_addressable() {
    for demo in DemoData::ALL {
        let plan = plan_for(demo);
        for (index, operator) in plan.operators.iter().enumerate() {
            assert_eq!(operator.id, index, "{demo:?} operator id/index contract");
        }
    }
}

#[test]
fn generated_standard_orders_match_demo_contract() {
    let plan = plan_for(DemoData::Standard);
    assert_eq!(plan.work_orders.len(), 155);

    for (process, count) in [(Quenching, 38), (Tempering, 36), (Nitriding, 4)] {
        assert_eq!(
            plan.work_orders
                .iter()
                .filter(|work_order| work_order.process == process)
                .count(),
            count,
            "{process:?} count"
        );
    }
    assert_eq!(
        plan.work_orders
            .iter()
            .filter(|work_order| work_order.priority == PriorityBand::Express)
            .count(),
        8
    );
    assert_eq!(
        plan.work_orders
            .iter()
            .filter(|work_order| work_order.priority == PriorityBand::Urgent)
            .count(),
        24
    );
}

#[test]
fn generated_standard_demo_covers_all_seven_visible_days() {
    let plan = plan_for(DemoData::Standard);
    let covered_days: std::collections::BTreeSet<_> = plan
        .shift_coverage_demands
        .iter()
        .filter(|demand| demand.roster_day >= 0)
        .map(|demand| demand.roster_day)
        .collect();

    assert_eq!(
        covered_days.into_iter().collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5, 6]
    );
}

#[test]
fn generated_process_envelopes_match_prd_defaults() {
    let plan = build_demo_plan();

    for work_order in &plan.work_orders {
        match work_order.process {
            StressRelieving => assert!(work_order.load_weight_kg <= 2000),
            Nitriding => assert!(work_order.soak_time_minutes >= 1200),
            Carburizing => assert!(work_order.soak_time_minutes >= 300),
            Aging => assert!((120..=220).contains(&work_order.temperature_celsius)),
            _ => {}
        }
    }
}

#[test]
fn demo_problem_starts_with_unassigned_work_orders() {
    for demo in DemoData::ALL {
        let plan = build_demo_plan_for(demo);
        assert_eq!(
            plan.assignments
                .iter()
                .filter(|assignment| assignment.assignment.is_some())
                .count(),
            0,
            "{demo:?} scheduled furnace assignments"
        );
        assert_eq!(
            plan.assignments
                .iter()
                .filter(|assignment| {
                    assignment.load_build_operator_id.is_some()
                        || assignment.program_operator_id.is_some()
                        || assignment.quench_operator_id.is_some()
                        || assignment.unload_operator_id.is_some()
                })
                .count(),
            0,
            "{demo:?} assigned task operators"
        );
        assert_eq!(
            plan.operator_shift_assignments
                .iter()
                .filter(|assignment| assignment.shift_type_enum().is_some())
                .count(),
            0,
            "{demo:?} working shift assignments"
        );
        assert!(plan
            .assignments
            .iter()
            .all(|assignment| assignment.assignment.is_none()));
        assert!(plan
            .assignments
            .iter()
            .all(|assignment| !assignment.compatible_assignments.is_empty()));
        assert!(plan.assignments.iter().any(|assignment| {
            !assignment.eligible_load_build_operator_ids.is_empty()
                && !assignment.eligible_program_operator_ids.is_empty()
                && !assignment.eligible_unload_operator_ids.is_empty()
        }));
        assert!(plan
            .operator_shift_assignments
            .iter()
            .all(|assignment| !assignment.allowed_shift_types.is_empty()));
        assert!(!plan.shift_coverage_demands.is_empty());
        assert!(plan
            .shift_coverage_demands
            .iter()
            .all(|demand| plan.shifts.get(demand.shift_id).is_some()));
    }
}

#[test]
fn demo_problem_assignment_ranges_are_complete_legal_domains() {
    let plan = build_demo_problem();

    for assignment in &plan.assignments {
        let work_order = plan.work_order(assignment.work_order_idx);
        let expected_compatible_furnaces = plan
            .furnaces
            .iter()
            .filter(|furnace| furnace_accepts_work_order(furnace, work_order))
            .count();
        let expected_slots = (0..NUM_TIME_SLOTS)
            .filter(|slot| {
                let start = slot * TIME_STEP;
                let end = start + work_order.soak_time_minutes as usize;
                end <= crate::domain::HORIZON_MINUTES
                    && slot_preserves_shift_ownership(
                        start,
                        work_order.soak_time_minutes as usize,
                        work_order.requires_quench,
                    )
            })
            .count();
        let expected_values = expected_compatible_furnaces * expected_slots;

        assert_eq!(
            assignment.compatible_assignments.len(),
            expected_values,
            "assignment range for work order {} must contain every legal furnace-slot pair",
            assignment.work_order_idx
        );

        for furnace in plan
            .furnaces
            .iter()
            .filter(|furnace| furnace_accepts_work_order(furnace, work_order))
        {
            assert!(
                assignment
                    .compatible_assignments
                    .iter()
                    .any(|value| value / NUM_TIME_SLOTS == furnace.id),
                "work order {} is missing legal furnace {}",
                assignment.work_order_idx,
                furnace.id
            );
        }
    }
}

#[test]
fn demo_problem_assignment_values_respect_physical_and_horizon_legality() {
    let plan = build_demo_problem();

    for assignment in &plan.assignments {
        for value in &assignment.compatible_assignments {
            let furnace_idx = value / NUM_TIME_SLOTS;
            let slot = value % NUM_TIME_SLOTS;
            let furnace = plan.furnace(furnace_idx);
            let start = slot * TIME_STEP;
            let end = start + assignment.duration_minutes();

            assert!(furnace.supports_process(assignment.process));
            assert!(furnace.supports_temperature(assignment.temperature_celsius));
            assert!(furnace.supports_load(assignment.load_weight_kg));
            assert!(end <= crate::domain::HORIZON_MINUTES);
            assert!(slot_preserves_shift_ownership(
                start,
                assignment.duration_minutes(),
                assignment.requires_quench,
            ));
        }
    }
}

#[test]
fn demo_problem_operator_ranges_are_non_empty_for_required_tasks() {
    let plan = build_demo_problem();

    for assignment in &plan.assignments {
        assert!(!assignment.eligible_load_build_operator_ids.is_empty());
        assert!(!assignment.eligible_program_operator_ids.is_empty());
        assert!(!assignment.eligible_unload_operator_ids.is_empty());
        if assignment.requires_quench {
            assert!(!assignment.eligible_quench_operator_ids.is_empty());
        } else {
            assert!(assignment.eligible_quench_operator_ids.is_empty());
        }
    }
}

#[test]
fn first_compatible_assignment_values_rotate_across_furnaces() {
    let plan = build_demo_problem();
    let mut first_furnace_counts: BTreeMap<usize, usize> = BTreeMap::new();
    for assignment in &plan.assignments {
        let Some(first_value) = assignment.compatible_assignments.first() else {
            continue;
        };
        *first_furnace_counts
            .entry(first_value / NUM_TIME_SLOTS)
            .or_default() += 1;
    }

    assert!(first_furnace_counts.len() > 1);
    let furnace_zero_count = *first_furnace_counts.get(&0).unwrap_or(&0);
    assert!(
        furnace_zero_count * 2 < plan.assignments.len(),
        "first compatible values should not concentrate on furnace 0"
    );
}

#[test]
fn demo_profiles_have_furnace_capacity_certificate() {
    const FURNACE_HARD_CONSTRAINTS: &[&str] = &[
        "unassignedOrders",
        "incompatibleProcess",
        "excessiveTemperature",
        "excessiveLoad",
        "outsideWeek",
        "furnaceOverlap",
        "changeoverGap",
        "loadBuildShiftOwnership",
        "programShiftOwnership",
        "quenchShiftOwnership",
        "unloadShiftOwnership",
    ];

    for demo in DemoData::ALL {
        let mut plan = build_demo_plan_for(demo);
        assign_greedy_furnace_certificate(&mut plan);
        for result in create_constraints().evaluate_each(&plan) {
            if FURNACE_HARD_CONSTRAINTS.contains(&result.name) {
                assert_eq!(result.score.hard(), 0, "{demo:?} {}", result.name);
            }
        }
    }
}

fn assign_greedy_furnace_certificate(plan: &mut crate::domain::Plan) {
    let mut order: Vec<usize> = (0..plan.assignments.len()).collect();
    order.sort_by_key(|index| {
        let assignment = &plan.assignments[*index];
        (
            assignment.due_datetime_minutes,
            std::cmp::Reverse(assignment.duration_minutes()),
            assignment.work_order_idx,
        )
    });

    let mut scheduled_by_furnace: BTreeMap<usize, Vec<ScheduledInterval>> = BTreeMap::new();
    for assignment_idx in order {
        let mut chosen = None;
        let mut values = plan.assignments[assignment_idx]
            .compatible_assignments
            .clone();
        values.sort_by_key(|value| {
            (
                (value % NUM_TIME_SLOTS) * TIME_STEP,
                value / NUM_TIME_SLOTS,
                *value,
            )
        });
        for value in values {
            let furnace_idx = value / NUM_TIME_SLOTS;
            let start = (value % NUM_TIME_SLOTS) * TIME_STEP;
            let end = start + plan.assignments[assignment_idx].duration_minutes();
            if end > HORIZON_MINUTES {
                continue;
            }
            if interval_fits_certificate(
                plan,
                assignment_idx,
                start,
                end,
                scheduled_by_furnace
                    .get(&furnace_idx)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
            ) {
                chosen = Some((value, furnace_idx, start, end));
                break;
            }
        }

        let (value, furnace_idx, start, end) = chosen.unwrap_or_else(|| {
            panic!(
                "no furnace certificate slot for work order {}",
                plan.assignments[assignment_idx].work_order_idx
            )
        });
        plan.assignments[assignment_idx].assignment = Some(value);
        scheduled_by_furnace
            .entry(furnace_idx)
            .or_default()
            .push(ScheduledInterval {
                assignment_idx,
                start,
                end,
            });
    }
}

fn interval_fits_certificate(
    plan: &crate::domain::Plan,
    assignment_idx: usize,
    start: usize,
    end: usize,
    scheduled: &[ScheduledInterval],
) -> bool {
    scheduled.iter().all(|other| {
        let other_assignment = &plan.assignments[other.assignment_idx];
        let assignment = &plan.assignments[assignment_idx];
        if end <= other.start {
            let required_gap =
                calculate_changeover_cost_for_sequence(assignment, other_assignment) as usize;
            other.start - end >= required_gap
        } else if other.end <= start {
            let required_gap =
                calculate_changeover_cost_for_sequence(other_assignment, assignment) as usize;
            start - other.end >= required_gap
        } else {
            false
        }
    })
}

#[test]
fn construction_order_hooks_are_deterministic_read_only_keys() {
    let plan = build_demo_problem();
    let assignment = plan
        .assignments
        .iter()
        .find(|assignment| !assignment.compatible_assignments.is_empty())
        .expect("demo has schedulable assignments");
    let value = assignment.compatible_assignments[0];
    let shift_assignment = plan
        .operator_shift_assignments
        .iter()
        .find(|assignment| !assignment.allowed_shift_types.is_empty())
        .expect("demo has shift assignments");
    let shift_value = shift_assignment.allowed_shift_types[0];

    assert_eq!(
        furnace_assignment_construction_entity_order_key(&plan, assignment),
        furnace_assignment_construction_entity_order_key(&plan, assignment)
    );
    assert_eq!(
        furnace_assignment_construction_value_order_key(&plan, assignment, value),
        furnace_assignment_construction_value_order_key(&plan, assignment, value)
    );
    assert_eq!(
        operator_shift_construction_entity_order_key(&plan, shift_assignment),
        operator_shift_construction_entity_order_key(&plan, shift_assignment)
    );
    assert_eq!(
        operator_shift_construction_value_order_key(&plan, shift_assignment, shift_value),
        operator_shift_construction_value_order_key(&plan, shift_assignment, shift_value)
    );
}
