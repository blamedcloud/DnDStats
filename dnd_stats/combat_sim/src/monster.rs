use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
use character_builder::basic_attack::BasicAttack;
use combat_core::actions::{ActionManager, ActionName, ActionType, AttackType, CombatAction, CombatOption};
use combat_core::damage::DamageType;
use combat_core::participant::Participant;
use combat_core::resources::{create_basic_rm, ResourceManager};
use rand_var::rv_traits::prob_type::RVProb;
use crate::CSError;

#[derive(Debug, Clone)]
pub struct Monster<T: RVProb> {
    max_hp: isize,
    ac: isize,
    resistances: HashSet<DamageType>,
    action_manager: ActionManager<T, CSError>,
    resource_manager: ResourceManager,
}

impl<T: RVProb> Monster<T> {
    pub fn new(max_hp: isize, ac: isize, ba: BasicAttack, num_attacks: u8) -> Self {
        Self {
            max_hp,
            ac,
            resistances: HashSet::new(),
            action_manager: create_basic_attack_am(ba, num_attacks),
            resource_manager: create_basic_rm(),
        }
    }
}

pub fn create_basic_attack_am<T: RVProb>(ba: BasicAttack, num_attacks: u8) -> ActionManager<T, CSError> {
    let mut am = ActionManager::new();
    am.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(num_attacks)));

    let pa_co = CombatOption::new_target(ActionType::SingleAttack, CombatAction::Attack(Rc::new(ba)), true);
    am.insert(ActionName::PrimaryAttack(AttackType::Normal), pa_co);

    am
}

impl<T: RVProb + Debug> Participant<T, CSError> for Monster<T> {
    fn get_ac(&self) -> isize {
        self.ac
    }

    fn get_max_hp(&self) -> isize {
        self.max_hp
    }

    fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
    }

    fn get_action_manager(&self) -> &ActionManager<T, CSError> {
        &self.action_manager
    }

    fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }
}
