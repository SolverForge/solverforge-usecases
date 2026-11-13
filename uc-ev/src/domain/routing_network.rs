//! Prepared road data the hot scoring path reads.
//!
//! Road travel is computed once per plan by `prepare.rs` (with
//! `solverforge-maps`) and frozen here. Constraints, route shadows, and the
//! list-variable hooks then read plain integer matrices instead of touching
//! the road network during every move.
//!
//! Node layout shared by every matrix in this file:
//! `[customers..., depots..., chargers...]`, where depots are the distinct
//! depot locations (vans that share a yard share one depot node).

use std::sync::Arc;

use solverforge_maps::UNREACHABLE;

use super::charging::{LegChargerOption, LegInput};

/// Candidate chargers kept per leg: the quickest detour, the charger nearest
/// the leg start, and the charger nearest the leg end. Keeping the two
/// nearest ones lets the planner rescue a nearly empty battery even when the
/// quickest detour is too far away to reach.
const OPTIONS_PER_LEG: usize = 3;

/// Customer terms copied out of `Plan.customers` so a vehicle can score its
/// own route without access to the whole plan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CustomerTerms {
    pub demand: i32,
    pub min_start_time: i64,
    pub max_end_time: i64,
    pub service_duration: i64,
}

/// Frozen road matrices plus per-leg charger candidates.
#[derive(Debug, Default)]
pub struct RoutingNetwork {
    pub customers: Vec<CustomerTerms>,
    pub depot_count: usize,
    pub charger_count: usize,
    /// Road seconds `[from][to]`; [`UNREACHABLE`] marks a missing road path.
    pub seconds: Vec<Vec<i64>>,
    /// Road metres `[from][to]`; [`UNREACHABLE`] marks a missing road path.
    pub meters: Vec<Vec<i64>>,
    /// Candidate charger detours for every customer/depot leg `[from][to]`.
    leg_options: Vec<Vec<Vec<LegChargerOption>>>,
}

impl RoutingNetwork {
    /// Freezes matrices and precomputes charger candidates for each leg.
    pub fn new(
        customers: Vec<CustomerTerms>,
        depot_count: usize,
        charger_count: usize,
        seconds: Vec<Vec<i64>>,
        meters: Vec<Vec<i64>>,
    ) -> Self {
        let mut network = Self {
            customers,
            depot_count,
            charger_count,
            seconds,
            meters,
            leg_options: Vec::new(),
        };
        let stops = network.customers.len() + depot_count;
        network.leg_options = (0..stops)
            .map(|from| (0..stops).map(|to| network.candidates(from, to)).collect())
            .collect();
        network
    }

    /// Matrix node of a depot location.
    pub fn depot_node(&self, depot_idx: usize) -> usize {
        self.customers.len() + depot_idx
    }

    /// Matrix node of a charger.
    pub fn charger_node(&self, charger_idx: usize) -> usize {
        self.customers.len() + self.depot_count + charger_idx
    }

    /// Whether both legs of a road pair exist.
    pub fn reachable(&self, from: usize, to: usize) -> bool {
        self.seconds[from][to] != UNREACHABLE && self.meters[from][to] != UNREACHABLE
    }

    /// Planner input for the leg `from -> to`.
    pub fn leg_input(&self, from: usize, to: usize) -> LegInput<'_> {
        let reachable = self.reachable(from, to);
        LegInput {
            direct_m: if reachable { self.meters[from][to] } else { 0 },
            direct_s: if reachable { self.seconds[from][to] } else { 0 },
            options: &self.leg_options[from][to],
        }
    }

    fn candidates(&self, from: usize, to: usize) -> Vec<LegChargerOption> {
        if from == to {
            return Vec::new();
        }
        let all: Vec<LegChargerOption> = (0..self.charger_count)
            .filter_map(|charger| {
                let node = self.charger_node(charger);
                (self.reachable(from, node) && self.reachable(node, to)).then(|| {
                    LegChargerOption {
                        charger,
                        to_charger_m: self.meters[from][node],
                        from_charger_m: self.meters[node][to],
                        to_charger_s: self.seconds[from][node],
                        from_charger_s: self.seconds[node][to],
                    }
                })
            })
            .collect();

        let mut picked: Vec<LegChargerOption> = Vec::with_capacity(OPTIONS_PER_LEG);
        let keys: [fn(&LegChargerOption) -> i64; OPTIONS_PER_LEG] = [
            |o| o.to_charger_s + o.from_charger_s,
            |o| o.to_charger_m,
            |o| o.from_charger_m,
        ];
        for key in keys {
            if let Some(best) = all.iter().min_by_key(|o| (key(o), o.charger)) {
                if !picked.iter().any(|o| o.charger == best.charger) {
                    picked.push(*best);
                }
            }
        }
        picked
    }
}

/// Shared handle compared by identity.
///
/// SolverForge derives `Eq` and `Hash` on planning entities. Comparing two
/// prepared networks element by element would be both slow and meaningless,
/// because one plan is only ever prepared against one network; pointer
/// identity is the correct equality for this cache.
#[derive(Clone, Debug)]
pub struct NetworkHandle(pub Arc<RoutingNetwork>);

impl PartialEq for NetworkHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NetworkHandle {}

impl std::hash::Hash for NetworkHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

impl std::ops::Deref for NetworkHandle {
    type Target = RoutingNetwork;

    fn deref(&self) -> &RoutingNetwork {
        &self.0
    }
}
