use std::collections::HashSet;
use std::fmt::Debug;
use combat_core::actions::ActionManager;
use combat_core::damage::DamageType;
use combat_core::participant::Participant;
use combat_core::resources::ResourceManager;
use rand_var::rv_traits::prob_type::RVProb;

#[derive(Debug, Clone)]
pub struct TargetDummy<T: RVProb> {
    max_hp: isize,
    ac: isize,
    resistances: HashSet<DamageType>,
    action_manager: ActionManager<T>,
    resource_manager: ResourceManager,
}

impl<T: RVProb> TargetDummy<T> {
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

impl<T: RVProb> Participant<T> for TargetDummy<T> {
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
}
