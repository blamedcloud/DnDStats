use std::collections::HashSet;
use character_builder::combat::ActionManager;
use character_builder::damage::DamageType;
use crate::participant::Participant;

#[derive(Clone)]
pub struct Monster {
    max_hp: isize,
    ac: isize,
    resistances: HashSet<DamageType>,
    action_manager: ActionManager,
}






impl Participant for Monster {
    fn get_ac(&self) -> isize {
        self.ac
    }

    fn get_max_hp(&self) -> isize {
        self.max_hp
    }

    fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
    }

    fn get_action_manager(&self) -> &ActionManager {
        &self.action_manager
    }
}
