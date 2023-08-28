use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

use character_builder::Character;
use combat_core::actions::{ActionManager, CABuilder, CombatAction, CombatOption};
use combat_core::damage::DamageType;
use combat_core::participant::Participant;
use combat_core::resources::ResourceManager;
use combat_core::triggers::TriggerManager;
use rand_var::rv_traits::prob_type::RVProb;

#[derive(Debug, Clone)]
pub struct Player<T: RVProb> {
    name: String,
    ac: isize,
    max_hp: isize,
    resistances: HashSet<DamageType>,
    action_manager: ActionManager<T>,
    resource_manager: ResourceManager,
    trigger_manager: TriggerManager,
}

impl<T: RVProb> Player<T> {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl<T: RVProb> From<Character> for Player<T> {
    fn from(value: Character) -> Self {
        let mut am = ActionManager::new();

        for (an, co) in value.get_action_builder().clone() {
            let cab = co.action;
            let at = co.action_type;
            let req_t = co.req_target;
            let ca: CombatAction<T> = match cab {
                CABuilder::WeaponAttack(wa) => CombatAction::Attack(Rc::new(wa)),
                CABuilder::SelfHeal(de) => CombatAction::SelfHeal(Rc::new(de)),
                CABuilder::AdditionalAttacks(aa) => CombatAction::AdditionalAttacks(aa),
                CABuilder::ByName => CombatAction::ByName,
            };
            let co = CombatOption::new_target(at, ca, req_t);
            am.insert(an, co);
        }

        Self {
            name: value.get_name().to_string(),
            ac: value.get_ac() as isize,
            max_hp: value.get_max_hp(),
            resistances: value.get_resistances().clone(),
            action_manager: am,
            resource_manager: value.get_resource_manager().clone(),
            trigger_manager: value.get_trigger_manager().clone(),
        }
    }
}

impl<T: RVProb> Participant<T> for Player<T> {
    fn get_ac(&self) -> isize {
        self.ac
    }

    fn get_max_hp(&self) -> isize {
        self.max_hp
    }

    fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
    }

    fn get_action_manager(&self) -> &ActionManager<T> {
        &self.action_manager
    }

    fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    fn has_triggers(&self) -> bool {
        true
    }

    fn get_trigger_manager(&self) -> Option<&TriggerManager> {
        Some(&self.trigger_manager)
    }
}
