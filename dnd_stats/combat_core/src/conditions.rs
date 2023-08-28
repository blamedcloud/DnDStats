use crate::actions::ActionType;
use crate::attack::AttackHitType;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ConditionName {
    Concentration,
    Invisible,
    Prone,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ConditionEffect {
    AttackerMod(AttackHitType), // ~ "your attacks have advantage"
    AtkTargetedMod(AttackHitType), // ~ "attacks against you have advantage"
}

pub enum ConditionLifetime {
    Permanent,
    UntilSpendAT(ActionType),

}


