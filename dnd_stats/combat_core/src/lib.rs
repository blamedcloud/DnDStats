use std::cmp;
use std::ops::{Add, AddAssign};

use rand_var::RandomVariable;
use rand_var::rv_traits::{RandVar, RVError};
use rand_var::rv_traits::prob_type::RVProb;

pub mod ability_scores;
pub mod actions;
pub mod attack;
pub mod combat_event;
pub mod combat_state;
pub mod conditions;
pub mod damage;
pub mod health;
pub mod movement;
pub mod participant;
pub mod resources;
pub mod skills;
pub mod strategy;
pub mod transposition;
pub mod triggers;

#[derive(Debug, Clone)]
pub enum CCError {
    ParticipantSizeErr,
    PMPushAfterCompile,
    PMNotCompiled,
    SMNotCompiled,
    SMPushAfterCompile,
    UnknownCondition,
    NoWeaponSet,
    RVE(RVError),
    Other(String),
}

impl From<RVError> for CCError {
    fn from(value: RVError) -> Self {
        CCError::RVE(value)
    }
}

impl From<String> for CCError {
    fn from(value: String) -> Self {
        CCError::Other(value)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum D20RollType {
    Disadvantage,
    FixedNormal,
    Normal,
    Advantage,
    SuperAdvantage,
}

impl D20RollType {
    pub fn get_rv<T: RVProb>(&self, d20: &D20Type) -> RandomVariable<T> {
        let rv = d20.get_rv();
        match self {
            D20RollType::Disadvantage => rv.min_two_trials(),
            D20RollType::FixedNormal => rv,
            D20RollType::Normal => rv,
            D20RollType::Advantage => rv.max_two_trials(),
            D20RollType::SuperAdvantage => rv.max_three_trials(),
        }
    }

    pub fn choose_better(&self, other: &D20RollType) -> D20RollType {
        cmp::max(*self, *other)
    }
}

impl Add<D20RollType> for D20RollType {
    type Output = D20RollType;

    fn add(self, rhs: D20RollType) -> Self::Output {
        // case 1: any FixedNormal -> FixedNormal
        if self == D20RollType::FixedNormal || rhs == D20RollType::FixedNormal {
            return D20RollType::FixedNormal;
        }
        // case 2: any Normal -> other
        if self == D20RollType::Normal {
            return rhs;
        } else if rhs == D20RollType::Normal {
            return self;
        }
        // case 3: same -> same
        if self == rhs {
            return self;
        }
        // case 4: different, no normals
        if self == D20RollType::Disadvantage || rhs == D20RollType::Disadvantage {
            // any disadvantage implies the other is adv or super adv, so cancel
            D20RollType::FixedNormal
        } else {
            // no disadvantage implies adv and super adv, so super adv.
            D20RollType::SuperAdvantage
        }
    }
}
impl AddAssign<D20RollType> for D20RollType {
    fn add_assign(&mut self, rhs: D20RollType) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum D20Type {
    D20,
    D20R1,
    D20m10,
}

impl D20Type {
    pub fn get_rv<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            D20Type::D20 => RandomVariable::new_dice(20).unwrap(),
            D20Type::D20R1 => RandomVariable::new_dice_reroll(20, 1).unwrap(),
            D20Type::D20m10 => RandomVariable::new_dice(20).unwrap().cap_lb(10).unwrap(),
        }
    }
}

// I could just use a bool instead, but I want to a avoid
// errors like "did true mean pass or fail in this context?"
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum BinaryOutcome {
    Pass,
    Fail,
}

#[cfg(test)]
mod tests {
    use num::Rational64;

    use rand_var::rv_traits::RandVar;

    use crate::{D20RollType, D20Type};

    #[test]
    fn roll_type_add() {
        assert_eq!(D20RollType::FixedNormal, D20RollType::Disadvantage + D20RollType::Advantage);
        assert_eq!(D20RollType::FixedNormal, D20RollType::Disadvantage + D20RollType::SuperAdvantage);
        assert_eq!(D20RollType::SuperAdvantage, D20RollType::Advantage + D20RollType::SuperAdvantage);
        assert_eq!(D20RollType::SuperAdvantage, D20RollType::Normal + D20RollType::SuperAdvantage);
        assert_eq!(D20RollType::FixedNormal, D20RollType::FixedNormal + D20RollType::Disadvantage);
    }

    #[test]
    fn d20_type() {
        let reliable_talent = D20Type::D20m10.get_rv();
        assert_eq!(10, reliable_talent.lower_bound());
        assert_eq!(20, reliable_talent.upper_bound());
        assert_eq!(Rational64::new(10, 20), reliable_talent.pdf(10));
    }

    #[test]
    fn better_d20_roll() {
        assert_eq!(D20RollType::Advantage, D20RollType::Disadvantage.choose_better(&D20RollType::Advantage));
        assert_eq!(D20RollType::SuperAdvantage, D20RollType::SuperAdvantage.choose_better(&D20RollType::Advantage));
        assert_eq!(D20RollType::Normal, D20RollType::Normal.choose_better(&D20RollType::Disadvantage));
    }
}
