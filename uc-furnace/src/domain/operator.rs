use solverforge::prelude::*;

use super::enums::{OperatorRole, OperatorSkill};
use super::support::SkillMask;

#[problem_fact]
pub struct Operator {
    #[planning_id]
    pub id: usize,
    pub name: &'static str,
    pub role: OperatorRole,
    pub day_only: bool,
    pub skills: &'static [OperatorSkill],
    pub skill_mask: SkillMask,
}

impl Operator {
    pub fn finalize(&mut self) {}
}
