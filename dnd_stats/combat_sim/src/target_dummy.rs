use std::collections::HashSet;
use std::fmt::Debug;

use combat_core::ability_scores::{Ability, AbilityScores};
use combat_core::actions::ActionManager;
use combat_core::conditions::ConditionManager;
use combat_core::damage::DamageType;
use combat_core::participant::Participant;
use combat_core::resources::ResourceManager;
use combat_core::skills::SkillManager;
use combat_core::triggers::TriggerManager;

use crate::monster::{ability_scores_by_cr, ac_to_cr, prof_by_cr};

#[derive(Debug, Clone)]
pub struct TargetDummy {
    max_hp: isize,
    ac: isize,
    prof: isize,
    resistances: HashSet<DamageType>,
    ability_scores: AbilityScores,
    skill_manager: SkillManager,
    action_manager: ActionManager,
    resource_manager: ResourceManager,
    condition_manager: ConditionManager,
}

impl TargetDummy {
    pub fn new(hp: isize, ac: isize) -> Self {
        let cr = ac_to_cr(ac);
        Self {
            max_hp: hp,
            ac,
            prof: prof_by_cr(cr),
            resistances: HashSet::new(),
            ability_scores: ability_scores_by_cr(cr, Ability::STR),
            skill_manager: SkillManager::new(),
            action_manager: ActionManager::new(),
            resource_manager: ResourceManager::just_action_types(),
            condition_manager: ConditionManager::new(),
        }
    }

    pub fn resistant(hp: isize, ac: isize, resistances: HashSet<DamageType>) -> Self {
        let cr = ac_to_cr(ac);
        Self {
            max_hp: hp,
            ac,
            prof: prof_by_cr(cr),
            resistances,
            ability_scores: ability_scores_by_cr(cr, Ability::STR),
            skill_manager: SkillManager::new(),
            action_manager: ActionManager::new(),
            resource_manager: ResourceManager::just_action_types(),
            condition_manager: ConditionManager::new(),
        }
    }
}

impl Participant for TargetDummy {
    fn get_ac(&self) -> isize {
        self.ac
    }

    fn get_max_hp(&self) -> isize {
        self.max_hp
    }

    fn get_prof(&self) -> isize {
        self.prof
    }

    fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
    }

    fn get_ability_scores(&self) -> &AbilityScores {
        &self.ability_scores
    }

    fn get_skill_manager(&self) -> &SkillManager {
        &self.skill_manager
    }

    fn get_action_manager(&self) -> &ActionManager {
        &self.action_manager
    }

    fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    fn has_triggers(&self) -> bool {
        false
    }

    fn get_trigger_manager(&self) -> Option<&TriggerManager> {
        None
    }

    fn get_condition_manager(&self) -> &ConditionManager {
        &self.condition_manager
    }
}
