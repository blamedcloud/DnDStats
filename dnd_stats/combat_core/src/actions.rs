use std::collections::HashMap;
use std::rc::Rc;
use rand_var::rv_traits::prob_type::RVProb;
use crate::attack::Attack;
use crate::damage::{DamageRV, DamageTerm};

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

#[derive(Debug, Clone)]
pub enum CABuilder<A, D> {
    WeaponAttack(A),
    SelfHeal(D),
    BonusDamage(DamageTerm),
    AdditionalAttacks(u8),
    ByName,
}

#[derive(Debug, Clone)]
pub enum CombatAction<T: RVProb, E> {
    Attack(Rc<dyn Attack<T, E>>),
    SelfHeal(Rc<dyn DamageRV<T, E>>),
    BonusDamage(DamageTerm),
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

impl<T: RVProb, E> From<(ActionType, CombatAction<T, E>)> for CombatOption<CombatAction<T, E>> {
    fn from(value: (ActionType, CombatAction<T, E>)) -> Self {
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

pub type ActionBuilder<A, D> = HashMap<ActionName, CombatOption<CABuilder<A, D>>>;
pub type ActionManager<T, E> = HashMap<ActionName, CombatOption<CombatAction<T, E>>>;
