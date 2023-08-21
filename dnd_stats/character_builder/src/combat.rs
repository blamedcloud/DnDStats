use std::collections::HashMap;
use crate::Character;
use crate::combat::attack::WeaponAttack;

pub mod attack;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ActionType {
    Action,
    SingleAttack,
    BonusAction,
    Reaction,
    Movement,
    FreeAction,
}

#[derive(Clone)]
pub enum CombatAction {
    Attack(WeaponAttack),
    AdditionalAttacks(u8),
    ByName,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum AttackType {
    Normal,
    GWMAttack,
    SSAttack,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ActionName {
    PrimaryAttack(AttackType),
    OffhandAttack(AttackType),
    BonusPAMAttack(AttackType),
    AttackAction,
    BonusGWMAttack,
    SecondWind,
    ActionSurge,
    Indomitable,
}

#[derive(Clone)]
pub struct CombatOption {
    pub action_type: ActionType,
    pub action: CombatAction,
}

impl CombatOption {
    pub fn new(at: ActionType, ca: CombatAction) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
        }
    }
}

impl From<(ActionType, CombatAction)> for CombatOption {
    fn from(value: (ActionType, CombatAction)) -> Self {
        CombatOption {
            action_type: value.0,
            action: value.1,
        }
    }
}

pub type ActionManager = HashMap<ActionName, CombatOption>;

pub fn create_action_manager(character: &Character) -> ActionManager {
    let mut am = ActionManager::new();
    am.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(1)));
    am.insert(ActionName::PrimaryAttack(AttackType::Normal),
              CombatOption::new(ActionType::SingleAttack,
                                CombatAction::Attack(WeaponAttack::primary_weapon(character))));
    if let Some(wa) = WeaponAttack::offhand_weapon(character) {
        am.insert(ActionName::OffhandAttack(AttackType::Normal),
                  CombatOption::new(ActionType::BonusAction, CombatAction::Attack(wa)));
    }
    am
}
