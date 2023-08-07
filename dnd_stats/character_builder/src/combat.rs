use crate::combat::attack::WeaponAttack;

pub mod attack;

pub enum ActionType {
    Action,
    BonusAction,
    Reaction,
    Movement,
    FreeAction,
}

pub enum CombatOption {
    Attack(WeaponAttack),
}