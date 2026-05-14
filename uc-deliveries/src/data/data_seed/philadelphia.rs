use super::types::LocationData;

mod depots;
mod visits;
mod visits_extra;

pub(super) use depots::DEPOTS;
pub(super) const VISIT_GROUPS: &[&[LocationData]] = &[visits::VISITS, visits_extra::VISITS];
