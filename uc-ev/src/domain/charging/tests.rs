//! The planner is checked against an exhaustive search over every subset of
//! candidate stops, so "optimal" is a tested claim rather than a comment.

use super::*;

fn option(charger: usize, to_m: i64, from_m: i64) -> LegChargerOption {
    LegChargerOption {
        charger,
        to_charger_m: to_m,
        from_charger_m: from_m,
        to_charger_s: to_m / 10,
        from_charger_s: from_m / 10,
    }
}

/// Owned test leg; `inputs` borrows these into planner input.
struct Leg(i64, Vec<LegChargerOption>);

fn leg(direct_m: i64, options: Vec<LegChargerOption>) -> Leg {
    Leg(direct_m, options)
}

fn inputs(legs: &[Leg]) -> Vec<LegInput<'_>> {
    legs.iter()
        .map(|Leg(direct_m, options)| LegInput {
            direct_m: *direct_m,
            direct_s: direct_m / 10,
            options,
        })
        .collect()
}

/// Exhaustive reference: tries every set of (leg, option) stops with at most
/// one stop per leg and keeps the lexicographically cheapest cost.
fn brute_force(legs: &[LegInput<'_>], initial: i64, range: i64, recharge: i64) -> (i64, i64) {
    let choices: Vec<(usize, usize)> = legs
        .iter()
        .enumerate()
        .flat_map(|(l, leg)| (0..leg.options.len()).map(move |o| (l, o)))
        .collect();
    let mut best = (i64::MAX, i64::MAX);
    for mask in 0_u32..(1 << choices.len()) {
        let picked: Vec<(usize, usize)> = (0..choices.len())
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| choices[bit])
            .collect();
        if picked.windows(2).any(|w| w[0].0 == w[1].0) {
            continue;
        }
        let mut drained = 0;
        let mut shortfall = 0;
        let mut added = 0;
        let mut cap = initial.min(range);
        for (l, leg) in legs.iter().enumerate() {
            match picked.iter().find(|(pl, _)| *pl == l) {
                Some(&(_, o)) => {
                    let opt = leg.options[o];
                    drained += opt.to_charger_m;
                    shortfall += (drained - cap).max(0);
                    drained = opt.from_charger_m;
                    cap = range;
                    added += opt.to_charger_s + opt.from_charger_s - leg.direct_s + recharge;
                }
                None => drained += leg.direct_m,
            }
        }
        shortfall += (drained - cap).max(0);
        best = best.min((shortfall, added));
    }
    best
}

fn plan_cost(legs: &[LegInput<'_>], plan: &ChargingPlan, recharge: i64) -> i64 {
    plan.stops
        .iter()
        .map(|stop| {
            let leg = &legs[stop.leg];
            let opt = leg.options.iter().find(|o| o.charger == stop.charger).unwrap();
            opt.to_charger_s + opt.from_charger_s - leg.direct_s + recharge
        })
        .sum()
}

#[test]
fn short_route_needs_no_stop() {
    let legs = vec![leg(10_000, vec![option(0, 4_000, 7_000)]), leg(10_000, vec![])];
    let legs = inputs(&legs);
    let plan = plan_charging(&legs, 30_000, 50_000, 1_800);
    assert!(plan.stops.is_empty());
    assert_eq!(plan.shortfall_m, 0);
}

#[test]
fn long_route_charges_once_and_becomes_feasible() {
    let legs = vec![
        leg(30_000, vec![option(3, 20_000, 12_000)]),
        leg(30_000, vec![option(4, 25_000, 6_000)]),
        leg(30_000, vec![]),
    ];
    let legs = inputs(&legs);
    let plan = plan_charging(&legs, 40_000, 80_000, 1_800);
    assert_eq!(plan.shortfall_m, 0);
    assert_eq!(plan.stops.len(), 1);
}

#[test]
fn impossible_route_reports_the_smallest_shortfall() {
    let legs = vec![leg(90_000, vec![]), leg(10_000, vec![])];
    let legs = inputs(&legs);
    let plan = plan_charging(&legs, 50_000, 60_000, 1_800);
    assert!(plan.stops.is_empty());
    assert_eq!(plan.shortfall_m, 50_000);
}

#[test]
fn initial_charge_above_range_is_clamped() {
    let legs = vec![leg(70_000, vec![])];
    let legs = inputs(&legs);
    let plan = plan_charging(&legs, 100_000, 60_000, 0);
    assert_eq!(plan.shortfall_m, 10_000);
}

#[test]
fn matches_exhaustive_search_on_generated_routes() {
    // Deterministic linear-congruential generator: no extra dependency and
    // the same 400 cases on every run.
    let mut seed = 0x2545_f491_u64;
    let mut next = |bound: i64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as i64) % bound
    };
    for _ in 0..400 {
        let leg_count = 1 + next(5) as usize;
        let owned: Vec<Leg> = (0..leg_count)
            .map(|_| {
                let direct = 5_000 + next(40_000);
                let option_count = next(3) as usize;
                let options = (0..option_count)
                    .map(|c| option(c, 1_000 + next(30_000), 1_000 + next(30_000)))
                    .collect();
                leg(direct, options)
            })
            .collect();
        let legs = inputs(&owned);
        let initial = next(60_000);
        let range = 30_000 + next(50_000);
        let recharge = next(3_600);
        let plan = plan_charging(&legs, initial, range, recharge);
        let expected = brute_force(&legs, initial, range, recharge);
        assert_eq!(
            (plan.shortfall_m, plan_cost(&legs, &plan, recharge)),
            expected,
            "legs={legs:?} initial={initial} range={range}"
        );
    }
}
