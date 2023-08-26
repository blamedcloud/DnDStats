use std::cmp;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{Display, Formatter};
use num::{BigRational, Rational64};
use rand_var::rv_traits::{RandVar, sequential};
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::sequential::{Pair, Seq, SeqIter};
use crate::CBError;
use crate::damage::DamageType;

pub mod weapon_attack;
pub mod basic_attack;

#[derive(Debug, Copy, Clone)]
pub enum AttackHitType {
    Disadvantage,
    Normal,
    Advantage,
    SuperAdvantage,
}

impl AttackHitType {
    pub fn get_rv<T: RVProb>(&self, d20: &D20Type) -> RandomVariable<T> {
        let rv = d20.get_rv();
        match self {
            AttackHitType::Disadvantage => rv.min_two_trials(),
            AttackHitType::Normal => rv,
            AttackHitType::Advantage => rv.max_two_trials(),
            AttackHitType::SuperAdvantage => rv.max_three_trials(),
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum D20Type {
    D20,
    D20R1,
}

impl D20Type {
    pub fn get_rv<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            D20Type::D20 => RandomVariable::new_dice(20).unwrap(),
            D20Type::D20R1 => RandomVariable::new_dice_reroll(20, 1).unwrap()
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum AttackResult {
    Miss,
    Hit,
    Crit,
}

// RollPair = Pair<roll, roll + bonus>
pub type RollPair = Pair<isize, isize>;

impl AttackResult {
    pub fn from(roll_pair: RollPair, ac: isize, crit_lb: isize) -> Self {
        let roll = roll_pair.0;
        let total = roll_pair.1;
        if roll == 20 {
            AttackResult::Crit
        } else if roll == 1 {
            AttackResult::Miss
        } else {
            if total >= ac {
                if roll >= crit_lb {
                    AttackResult::Crit
                } else {
                    AttackResult::Hit
                }
            } else {
                AttackResult::Miss
            }
        }
    }
}

impl Display for AttackResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        match self {
            AttackResult::Crit => s.push_str("Crit"),
            AttackResult::Hit => s.push_str("Hit"),
            AttackResult::Miss => s.push_str("Miss"),
        };
        write!(f, "{}", s)
    }
}

impl Seq for AttackResult {
    // I'm sure there's a better way to do this, but idk
    fn gen_seq(&self, other: &Self) -> SeqIter<Self> {
        let first = *cmp::min(self, other);
        let second = *cmp::max(self, other);
        let arr = [AttackResult::Miss, AttackResult::Hit, AttackResult::Crit];
        let iter= arr.iter().filter(|ar| (*ar >= &first) && (*ar <= &second));
        let items: BTreeSet<AttackResult> = iter.copied().collect();
        SeqIter { items }
    }

    fn always_convex() -> bool {
        true
    }

    fn convex_bounds(iter: SeqIter<Self>) -> Option<(Self, Self)> {
        sequential::always_convex_bounds(iter)
    }
}

pub type AccMRV<T> = MapRandVar<RollPair, T>;
pub type AccMRV64 = MapRandVar<RollPair, Rational64>;
pub type AccMRVBig = MapRandVar<RollPair, BigRational>;

pub type ArMRV<T> = MapRandVar<AttackResult, T>;
pub type ArMRV64 = MapRandVar<AttackResult, Rational64>;
pub type ArMRVBig = MapRandVar<AttackResult, BigRational>;

pub type AoMRV<T> = MapRandVar<Pair<AttackResult, isize>, T>;
pub type AoMRV64 = MapRandVar<Pair<AttackResult, isize>, Rational64>;
pub type AoMRVBig = MapRandVar<Pair<AttackResult, isize>, BigRational>;

pub trait Attack {
    fn get_dmg_map<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<BTreeMap<AttackResult, RandomVariable<T>>, CBError>;
    fn get_accuracy_rv<T: RVProb>(&self, hit_type: AttackHitType) -> Result<AccMRV<T>, CBError>;

    fn get_crit_lb(&self) -> isize {
        20
    }

    fn get_attack_result_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize) -> Result<ArMRV<T>, CBError> {
        let hit_rv = self.get_accuracy_rv(hit_type)?;
        Ok(hit_rv.map_keys(|hit| AttackResult::from(hit, target_ac, self.get_crit_lb())))
    }

    fn get_attack_dmg_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        let attack_result_rv = self.get_attack_result_rv(hit_type, target_ac)?;
        let dmg_map = self.get_dmg_map(resistances)?;
        Ok(attack_result_rv.consolidate(&dmg_map)?.into())
    }

    fn get_attack_outcome_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<AoMRV<T>, CBError> {
        let attack_result_rv = self.get_attack_result_rv(hit_type, target_ac)?;
        let dmg_map = self.get_dmg_map(resistances)?;
        Ok(attack_result_rv.projection(&dmg_map)?)
    }
}
