use std::collections::HashMap;
use crate::attack::basic_attack::BasicAttack;
use crate::conditions::{Condition, ConditionName};
use crate::damage::dice_expr::DiceExpression;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ActionType {
    Action,
    SingleAttack,
    BonusAction,
    Reaction,
    Movement,
    HalfMove,
    FreeAction,
}

impl ActionType {
    pub fn iterator() -> impl Iterator<Item = ActionType> {
        [
            ActionType::Action,
            ActionType::SingleAttack,
            ActionType::BonusAction,
            ActionType::Reaction,
            ActionType::Movement,
            ActionType::HalfMove,
            ActionType::FreeAction,
        ].iter().copied()
    }
}

// TODO:
// either rename this one or parameterize the other one
// now that CombatAction isn't prob-parameterized, we can
// just make the ActionManager type know which attack and heal it uses.
#[derive(Debug, Clone)]
pub enum CABuilder<A, H> {
    WeaponAttack(A),
    SelfHeal(H),
    AdditionalAttacks(u8),
    ApplyCondition(ConditionName),
    ApplyComplexCondition(ConditionName, Condition),
    ByName,
}

#[derive(Debug, Clone)]
pub enum CombatAction {
    Attack(BasicAttack),
    SelfHeal(DiceExpression),
    AdditionalAttacks(u8),
    ApplyBasicCondition(ConditionName),
    ApplyComplexCondition(ConditionName, Condition),
    ByName,
}

#[derive(Debug, Clone)]
pub struct CombatOption<CA> {
    pub action_type: ActionType,
    pub action: CA,
    pub req_target: bool,
}

impl<CA> CombatOption<CA> {
    pub fn new(at: ActionType, ca: CA) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
            req_target: false
        }
    }

    pub fn new_target(at: ActionType, ca: CA, rt: bool) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
            req_target: rt
        }
    }
}

impl From<(ActionType, CombatAction)> for CombatOption<CombatAction> {
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
    Indomitable, // TODO: move to triggers (OnSave trigger)
    ShoveProne,
}

pub type ActionBuilder<A, H> = HashMap<ActionName, CombatOption<CABuilder<A, H>>>;
pub type ActionManager = HashMap<ActionName, CombatOption<CombatAction>>;
