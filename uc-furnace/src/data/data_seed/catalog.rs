use std::str::FromStr;

use crate::domain::{
    Furnace, FurnaceAssignment, Operator, Plan, Shift, ShiftCoverageDemand, WorkOrder,
};

use super::problem::build_demo_problem_for;

const ORDER_COUNTS: &[(usize, usize)] = &[
    (0, 38), // Quenching
    (1, 36), // Tempering
    (2, 15), // Annealing
    (3, 14), // Carburizing
    (4, 4),  // Nitriding
    (5, 18), // StressRelieving
    (6, 18), // Normalizing
    (7, 6),  // SolutionTreating
    (8, 6),  // Aging
];

const DUE_WINDOWS: &[usize] = &[
    1320, // Mon 22:00
    2280, // Tue 14:00
    2760, // Tue 22:00
    3720, // Wed 14:00
    4200, // Wed 22:00
    5160, // Thu 14:00
    5640, // Thu 22:00
    6600, // Fri 14:00
    7080, // Fri 22:00
    8040, // Sat 14:00
    8520, // Sat 22:00
    9480, // Sun 14:00
    9960, // Sun 22:00
];

const DUE_SLACK: &[usize] = &[0, 120, 240];

pub(super) struct DemoSeed {
    pub(super) furnaces: Vec<Furnace>,
    pub(super) work_orders: Vec<WorkOrder>,
    pub(super) operators: Vec<Operator>,
    pub(super) shifts: Vec<Shift>,
    pub(super) shift_coverage_demands: Vec<ShiftCoverageDemand>,
    pub(super) assignments: Vec<FurnaceAssignment>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DemoDataProfile {
    pub(super) seed: u64,
    pub(super) order_counts: &'static [(usize, usize)],
    pub(super) due_windows: &'static [usize],
    pub(super) due_slack_minutes: &'static [usize],
    pub(super) express_period: usize,
    pub(super) urgent_width: usize,
    pub(super) shift_lead_reserves: usize,
    pub(super) furnace_operator_reserves: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoDataSize {
    Standard,
}

impl FromStr for DemoDataSize {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "STANDARD" => Ok(Self::Standard),
            _ => Err(()),
        }
    }
}

impl DemoDataSize {
    pub const ALL: [Self; 1] = [Self::Standard];
    pub const DEFAULT: Self = Self::Standard;

    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
        }
    }

    pub(super) const fn profile(self) -> DemoDataProfile {
        match self {
            Self::Standard => DemoDataProfile {
                seed: 0x00DE_CAFB_AD17_A11A,
                order_counts: ORDER_COUNTS,
                due_windows: DUE_WINDOWS,
                due_slack_minutes: DUE_SLACK,
                express_period: 20,
                urgent_width: 3,
                shift_lead_reserves: 2,
                furnace_operator_reserves: 3,
            },
        }
    }

    pub fn order_count(self) -> usize {
        self.profile()
            .order_counts
            .iter()
            .map(|(_, count)| *count)
            .sum()
    }
}

pub type DemoData = DemoDataSize;

pub fn generate(demo: DemoData) -> Plan {
    build_demo_problem_for(demo)
}
