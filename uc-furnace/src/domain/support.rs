use super::constants::{RosterDay, ShiftId};
use super::enums::ShiftType;
use super::enums::{OperatorRole, OperatorSkill};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SkillMask(pub u16);

impl SkillMask {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn insert(self, skill: OperatorSkill) -> Self {
        Self(self.0 | skill.bit())
    }

    pub const fn contains(self, skill: OperatorSkill) -> bool {
        self.0 & skill.bit() != 0
    }

    pub const fn contains_all(self, required: SkillMask) -> bool {
        self.0 & required.0 == required.0
    }

    pub const fn from_slice(skills: &[OperatorSkill]) -> Self {
        let mut bits = 0u16;
        let mut idx = 0usize;
        while idx < skills.len() {
            bits |= skills[idx].bit();
            idx += 1;
        }
        Self(bits)
    }

    pub fn names(self) -> Vec<String> {
        OperatorSkill::ALL
            .into_iter()
            .filter(|skill| self.contains(*skill))
            .map(|skill| skill.to_string())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShiftIdentity {
    pub shift_id: ShiftId,
    pub roster_day: RosterDay,
    pub shift_type: ShiftType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManualTaskKind {
    LoadBuild,
    Program,
    Quench,
    Unload,
    HeavyUnload,
}

impl std::fmt::Display for ManualTaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LoadBuild => "LoadBuild",
            Self::Program => "Program",
            Self::Quench => "Quench",
            Self::Unload => "Unload",
            Self::HeavyUnload => "HeavyUnload",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ManualTaskDemand {
    pub work_order_idx: usize,
    pub furnace_idx: usize,
    pub shift: ShiftIdentity,
    pub kind: ManualTaskKind,
    pub start_minute: usize,
    pub end_minute: usize,
    pub required_skills: SkillMask,
}

impl ManualTaskDemand {
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start_minute < other.end_minute && other.start_minute < self.end_minute
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ManualTaskBooking {
    pub work_order_idx: usize,
    pub furnace_idx: usize,
    pub operator_id: usize,
    pub shift: ShiftIdentity,
    pub kind: ManualTaskKind,
    pub start_minute: usize,
    pub end_minute: usize,
    pub required_skills: SkillMask,
}

impl ManualTaskBooking {
    pub fn identity_key(&self) -> ManualTaskBookingIdentity {
        ManualTaskBookingIdentity {
            work_order_idx: self.work_order_idx,
            kind: self.kind,
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.start_minute < other.end_minute && other.start_minute < self.end_minute
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ManualTaskBookingIdentity {
    pub work_order_idx: usize,
    pub kind: ManualTaskKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NightWindowEntry {
    pub operator_id: usize,
    pub window_start_day: RosterDay,
    pub delta: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MonitoringLoad {
    pub ramp: i64,
    pub soak: i64,
    pub capacity: i64,
    pub task: i64,
}

impl MonitoringLoad {
    pub fn required_operators(self) -> i64 {
        ceil_div_nonnegative(self.ramp, 2) + ceil_div_nonnegative(self.soak, 4) + self.task.max(0)
    }
}

impl std::ops::AddAssign for MonitoringLoad {
    fn add_assign(&mut self, rhs: Self) {
        self.ramp += rhs.ramp;
        self.soak += rhs.soak;
        self.capacity += rhs.capacity;
        self.task += rhs.task;
    }
}

fn ceil_div_nonnegative(value: i64, divisor: i64) -> i64 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}

impl std::ops::SubAssign for MonitoringLoad {
    fn sub_assign(&mut self, rhs: Self) {
        self.ramp -= rhs.ramp;
        self.soak -= rhs.soak;
        self.capacity -= rhs.capacity;
        self.task -= rhs.task;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonitoringEntry {
    pub bucket: usize,
    pub load: MonitoringLoad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShiftWorkEntry {
    pub operator_id: usize,
    pub shift_id: ShiftId,
    pub delta: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShiftCoverageEntry {
    pub shift_id: ShiftId,
    pub role: OperatorRole,
    pub delta: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HardViolation {
    pub weight: i64,
}
