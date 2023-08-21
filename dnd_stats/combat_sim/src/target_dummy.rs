use std::collections::HashSet;
use character_builder::combat::ActionManager;
use character_builder::damage::DamageType;
use character_builder::resources::ResourceManager;
use crate::participant::Participant;

pub struct TargetDummy {
    max_hp: isize,
    ac: isize,
    resistances: HashSet<DamageType>,
    action_manager: ActionManager,
    resource_manager: ResourceManager,
}

impl TargetDummy {
    pub fn new(hp: isize, ac: isize) -> Self {
        TargetDummy {
            max_hp: hp,
            ac,
            resistances: HashSet::new(),
            action_manager: ActionManager::new(),
            resource_manager: ResourceManager::new(),
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

    fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
    }

    fn get_action_manager(&self) -> &ActionManager {
        &self.action_manager
    }

    fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    fn get_resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }
}
