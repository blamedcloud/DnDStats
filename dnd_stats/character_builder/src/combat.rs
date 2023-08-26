use std::collections::HashMap;
use crate::Character;
use crate::combat::attack::basic_attack::BasicAttack;
use crate::combat::attack::weapon_attack::WeaponAttack;
use crate::damage::{DamageTerm, DiceExpression};

pub mod attack;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ActionType {
    Action,
    SingleAttack,
    BonusAction,
    Reaction,
    Movement,
    FreeAction,
    OnHit,
}

#[derive(Clone)]
pub enum CombatAction {
    WeaponAttack(WeaponAttack),
    BasicAttack(BasicAttack),
    SelfHeal(DiceExpression),
    BonusDamage(DamageTerm),
    AdditionalAttacks(u8),
    ByName,
}

#[derive(Clone)]
pub struct CombatOption {
    pub action_type: ActionType,
    pub action: CombatAction,
    pub req_target: bool,
}

impl CombatOption {
    pub fn new(at: ActionType, ca: CombatAction) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
            req_target: false
        }
    }

    pub fn new_target(at: ActionType, ca: CombatAction, rt: bool) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
            req_target: rt
        }
    }
}

impl From<(ActionType, CombatAction)> for CombatOption {
    fn from(value: (ActionType, CombatAction)) -> Self {
        CombatOption {
            action_type: value.0,
            action: value.1,
            req_target: false,
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum AttackType {
    Normal,
    GWMAttack,
    SSAttack,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ActionName {
    PrimaryAttack(AttackType),
    OffhandAttack(AttackType),
    BonusPAMAttack(AttackType),
    AttackAction,
    BonusGWMAttack,
    SecondWind,
    ActionSurge,
    Indomitable,
    SneakAttack,
}

pub type ActionManager = HashMap<ActionName, CombatOption>;

pub fn create_basic_attack_am(ba: BasicAttack, num_attacks: u8) -> ActionManager {
    let mut am = ActionManager::new();
    am.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(num_attacks)));

    let pa_co = CombatOption::new_target(ActionType::SingleAttack, CombatAction::BasicAttack(ba), true);
    am.insert(ActionName::PrimaryAttack(AttackType::Normal), pa_co);

    am
}

pub fn create_character_am(character: &Character) -> ActionManager {
    let mut am = ActionManager::new();
    am.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(1)));

    let wa = WeaponAttack::primary_weapon(character);
    let pa_co = CombatOption::new_target(ActionType::SingleAttack, CombatAction::WeaponAttack(wa), true);
    am.insert(ActionName::PrimaryAttack(AttackType::Normal), pa_co);

    if let Some(owa) = WeaponAttack::offhand_weapon(character) {
        let oa_co = CombatOption::new_target(ActionType::BonusAction, CombatAction::WeaponAttack(owa), true);
        am.insert(ActionName::OffhandAttack(AttackType::Normal), oa_co);
    }
    am
}
