use std::collections::HashMap;
use crate::attack::basic_attack::BasicAttack;
use crate::conditions::{Condition, ConditionName};
use crate::damage::dice_expr::DiceExpression;
use crate::participant::ParticipantId;
use crate::resources::ResourceName;

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
pub enum CombatAction<A, DE> {
    Attack(A),
    SelfHeal(DE),
    GainResource(ResourceName, usize),
    ApplyBasicCondition(ConditionName),
    ApplyComplexCondition(ConditionName, Condition),
    ByName,
}

#[derive(Debug, Clone)]
pub struct CombatOption<A, DE> {
    pub action_type: ActionType,
    pub action: CombatAction<A, DE>,
    pub req_target: bool,
}

impl<A, DE> CombatOption<A, DE> {
    pub fn new(at: ActionType, ca: CombatAction<A, DE>) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
            req_target: false
        }
    }

    pub fn new_target(at: ActionType, ca: CombatAction<A, DE>, rt: bool) -> Self {
        CombatOption {
            action_type: at,
            action: ca,
            req_target: rt
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
    PlanarWarrior,
    FavoredFoeApply,
    FavoredFoeUse,
}

pub type ActionBuilder<A, DE> = HashMap<ActionName, CombatOption<A, DE>>;
pub type ActionManager = HashMap<ActionName, CombatOption<BasicAttack, DiceExpression>>;

pub fn register_pid(am: &mut ActionManager, pid: ParticipantId) {
    for (_, co) in am.iter_mut() {
        match &mut co.action {
            CombatAction::ApplyComplexCondition(_, cond) => {
                cond.register_pid(pid);
            },
            _ => {}
        }
    }
}
