use std::collections::HashMap;
use std::rc::Rc;

use rand_var::rv_traits::prob_type::RVProb;

use crate::attack::Attack;
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

#[derive(Debug, Clone)]
pub enum CABuilder<A, D> {
    WeaponAttack(A),
    SelfHeal(D),
    AdditionalAttacks(u8),
    ByName,
}

#[derive(Debug, Clone)]
pub enum CombatAction<T: RVProb> {
    Attack(Rc<dyn Attack<T>>),
    SelfHeal(DiceExpression),
    AdditionalAttacks(u8),
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

impl<T: RVProb> From<(ActionType, CombatAction<T>)> for CombatOption<CombatAction<T>> {
    fn from(value: (ActionType, CombatAction<T>)) -> Self {
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

pub type ActionBuilder<A, D> = HashMap<ActionName, CombatOption<CABuilder<A, D>>>;
pub type ActionManager<T> = HashMap<ActionName, CombatOption<CombatAction<T>>>;
