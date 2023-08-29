use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

use character_builder::basic_attack::BasicAttack;
use combat_core::ability_scores::{Ability, AbilityScores};
use combat_core::actions::{ActionManager, ActionName, ActionType, AttackType, CombatAction, CombatOption};
use combat_core::conditions::ConditionManager;
use combat_core::damage::DamageType;
use combat_core::participant::Participant;
use combat_core::resources::ResourceManager;
use combat_core::skills::SkillManager;
use combat_core::triggers::TriggerManager;
use rand_var::rv_traits::prob_type::RVProb;

#[derive(Debug, Clone)]
pub struct Monster<T: RVProb> {
    max_hp: isize,
    ac: isize,
    prof: isize,
    resistances: HashSet<DamageType>,
    ability_scores: AbilityScores,
    skill_manager: SkillManager,
    action_manager: ActionManager<T>,
    resource_manager: ResourceManager,
    condition_manager: ConditionManager,
}

impl<T: RVProb> Monster<T> {
    pub fn new(max_hp: isize, ac: isize, prof: isize, ba: BasicAttack, num_attacks: u8) -> Self {
        Self {
            max_hp,
            ac,
            prof,
            resistances: HashSet::new(),
            ability_scores: ability_scores_by_prof(prof as u8, Ability::STR),
            skill_manager: SkillManager::new(),
            action_manager: create_basic_attack_am(ba, num_attacks),
            resource_manager: ResourceManager::just_action_types(),
            condition_manager: ConditionManager::new(),
        }
    }
}

// finds the first cr that gives at least a given AC
pub fn ac_to_cr(ac: isize) -> u8 {
    if ac <= ac_by_cr(0) {
        return 0;
    }
    let mut cr = 1;
    for _ in 0..30 {
        if ac_by_cr(cr) == ac {
            return cr;
        }
        cr += 1;
    }
    cr
}

pub fn ac_by_cr(cr: u8) -> isize {
    // AC is mostly cr + prof with a few bumps.
    let mut ac = (cr as isize) + prof_by_cr(cr);
    if cr >= 4 {
        ac += 1;
    }
    if cr >= 8 {
        ac += 1;
    }
    if cr == 9 {
        ac -= 1;
    }
    ac
}

pub fn prof_by_cr(cr: u8) -> isize {
    if cr == 0 {
        2
    } else {
        (cr as isize + 3)/4 + 1
    }
}

pub fn ability_scores_by_prof(prof: u8, primary_ability: Ability) -> AbilityScores {
    let score = 8 + prof;
    let mut scores = AbilityScores::new(score, score, score, score, score, score);
    scores.get_score_mut(&primary_ability).set_prof_save(true);
    scores.constitution.set_prof_save(true);
    scores
}

pub fn ability_scores_by_cr(cr: u8, primary_ability: Ability) -> AbilityScores {
    let prof = prof_by_cr(cr) as u8;
    ability_scores_by_prof(prof, primary_ability)
}

pub fn create_basic_attack_am<T: RVProb>(ba: BasicAttack, num_attacks: u8) -> ActionManager<T> {
    let mut am = ActionManager::new();
    am.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(num_attacks)));

    let pa_co = CombatOption::new_target(ActionType::SingleAttack, CombatAction::Attack(Rc::new(ba)), true);
    am.insert(ActionName::PrimaryAttack(AttackType::Normal), pa_co);

    am
}

impl<T: RVProb> Participant<T> for Monster<T> {
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

    fn get_action_manager(&self) -> &ActionManager<T> {
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
