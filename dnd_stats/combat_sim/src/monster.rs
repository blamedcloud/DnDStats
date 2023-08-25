use std::collections::HashSet;
use character_builder::combat::{ActionManager, create_basic_attack_am};
use character_builder::combat::attack::basic_attack::BasicAttack;
use character_builder::damage::DamageType;
use crate::participant::Participant;

#[derive(Clone)]
pub struct Monster {
    max_hp: isize,
    ac: isize,
    resistances: HashSet<DamageType>,
    action_manager: ActionManager,
}

impl Monster {
    pub fn new(max_hp: isize, ac: isize, ba: BasicAttack, num_attacks: u8) -> Self {
        Self {
            max_hp,
            ac,
            resistances: HashSet::new(),
            action_manager: create_basic_attack_am(ba, num_attacks)
        }
    }
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
