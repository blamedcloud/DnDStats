use std::collections::HashMap;
use crate::Character;
use crate::combat::attack::WeaponAttack;
use crate::damage::DiceExpression;

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
    SelfHeal(DiceExpression),
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
}

pub type ActionManager = HashMap<ActionName, CombatOption>;

pub fn create_action_manager(character: &Character) -> ActionManager {
    let mut am = ActionManager::new();
    am.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(1)));

    let wa = WeaponAttack::primary_weapon(character);
    let pa_co = CombatOption::new_target(ActionType::SingleAttack, CombatAction::Attack(wa), true);
    am.insert(ActionName::PrimaryAttack(AttackType::Normal), pa_co);

    if let Some(owa) = WeaponAttack::offhand_weapon(character) {
        let oa_co = CombatOption::new_target(ActionType::BonusAction, CombatAction::Attack(owa), true);
        am.insert(ActionName::OffhandAttack(AttackType::Normal), oa_co);
    }
    am
}
