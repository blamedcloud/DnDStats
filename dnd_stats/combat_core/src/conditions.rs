use crate::actions::ActionType;
use crate::D20RollType;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ConditionName {
    Concentration,
    Invisible,
    Prone,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ConditionEffect {
    AttackerMod(D20RollType), // ~ "your attacks have advantage"
    AtkTargetedMod(D20RollType), // ~ "attacks against you have advantage"
}

pub enum ConditionLifetime {
    Permanent,
    UntilSpendAT(ActionType),

}


