use std::collections::HashMap;
use crate::Character;
use crate::combat::attack::WeaponAttack;

pub mod attack;

#[derive(Copy, Clone)]
pub enum ActionType {
    Action,
    BonusAction,
    Reaction,
    Movement,
    FreeAction,
}

#[derive(Clone)]
pub enum CombatAction {
    Attack(WeaponAttack),
    Hide,
    Dash,
    Disengage,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ActionNames {
    BasicAttack,
    OffhandAttack,
}

#[derive(Clone)]
pub struct CombatOption {
    pub action_type: ActionType,
    pub action: CombatAction,
}

// this will probably be replaced with a struct eventually
pub type ActionManager = HashMap<ActionNames, CombatOption>;

pub fn create_action_manager(character: &Character) -> ActionManager {
    let mut am = ActionManager::new();
    am.insert(ActionNames::BasicAttack,
              CombatOption {
                  action_type: ActionType::Action,
                  action: CombatAction::Attack(WeaponAttack::primary_weapon(character))
              });
    if let Some(wa) = WeaponAttack::offhand_weapon(character) {
        am.insert(ActionNames::OffhandAttack,
                  CombatOption {
                      action_type: ActionType::BonusAction,
                      action: CombatAction::Attack(wa)
                  });
    }
    am
}
