use std::cmp;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{Debug, Display, Formatter};

use num::{BigRational, Rational64};

use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::sequential;
use rand_var::rv_traits::sequential::{Pair, Seq, SeqIter};

use crate::{CCError, D20RollType};
use crate::combat_event::CombatEvent;
use crate::conditions::AttackDistance;
use crate::damage::{DamageTerm, DamageType};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone)]
pub struct AtkDmgMap<T: RVProb> {
    miss_dmg: RandomVariable<T>,
    hit_dmg: RandomVariable<T>,
    crit_dmg: RandomVariable<T>,
}

impl<T: RVProb> AtkDmgMap<T> {
    pub fn new(miss_dmg: RandomVariable<T>, hit_dmg: RandomVariable<T>, crit_dmg: RandomVariable<T>) -> Self {
        Self {
            miss_dmg,
            hit_dmg,
            crit_dmg,
        }
    }

    pub fn into_ar_map(self) -> BTreeMap<AttackResult, RandomVariable<T>> {
        let mut map = BTreeMap::new();
        map.insert(AttackResult::Miss, self.miss_dmg);
        map.insert(AttackResult::Hit, self.hit_dmg);
        map.insert(AttackResult::Crit, self.crit_dmg);
        map
    }

    pub fn into_ce_map(self) -> BTreeMap<CombatEvent, RandomVariable<T>> {
        let mut map = BTreeMap::new();
        map.insert(CombatEvent::AR(AttackResult::Miss), self.miss_dmg);
        map.insert(CombatEvent::AR(AttackResult::Hit), self.hit_dmg);
        map.insert(CombatEvent::AR(AttackResult::Crit), self.crit_dmg);
        map
    }
}

pub trait Attack<T: RVProb> : Debug {
    fn get_miss_dmg(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError>;
    fn get_hit_dmg(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError>;
    fn get_crit_dmg(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError>;
    fn get_acc_rv(&self, hit_type: D20RollType) -> Result<AccMRV<T>, CCError>;

    // TODO: this should eventually return something like Equipment::WeaponRange instead
    // and then the map should validate this.
    fn get_atk_range(&self) -> AttackDistance;

    fn get_ar_dmg(&self, ar: AttackResult, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        match ar {
            AttackResult::Miss => self.get_miss_dmg(resistances, bonus_dmg),
            AttackResult::Hit => self.get_hit_dmg(resistances, bonus_dmg),
            AttackResult::Crit => self.get_crit_dmg(resistances, bonus_dmg),
        }
    }

    fn get_dmg_map(&self, resistances: &HashSet<DamageType>) -> Result<AtkDmgMap<T>, CCError> {
        Ok(AtkDmgMap::new(
            self.get_miss_dmg(resistances, vec!())?,
            self.get_hit_dmg(resistances, vec!())?,
            self.get_crit_dmg(resistances, vec!())?
        ))
    }

    fn get_crit_lb(&self) -> isize {
        20
    }

    fn get_ar_rv(&self, hit_type: D20RollType, target_ac: isize) -> Result<ArMRV<T>, CCError> {
        let hit_rv = self.get_acc_rv(hit_type)?;
        Ok(hit_rv.map_keys(|hit| AttackResult::from(hit, target_ac, self.get_crit_lb())))
    }

    fn get_ce_rv(&self, hit_type: D20RollType, target_ac: isize) -> Result<MapRandVar<CombatEvent, T>, CCError> {
        let ar_rv = self.get_ar_rv(hit_type, target_ac)?;
        Ok(ar_rv.map_keys(|ar| ar.into()))
    }

    fn get_dmg_rv(&self, hit_type: D20RollType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CCError> {
        let attack_result_rv = self.get_ar_rv(hit_type, target_ac)?;
        let dmg_map = self.get_dmg_map(resistances)?;
        Ok(attack_result_rv.consolidate(&dmg_map.into_ar_map())?.into())
    }

    fn get_ao_rv(&self, hit_type: D20RollType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<AoMRV<T>, CCError> {
        let attack_result_rv = self.get_ar_rv(hit_type, target_ac)?;
        let dmg_map = self.get_dmg_map(resistances)?;
        Ok(attack_result_rv.projection(&dmg_map.into_ar_map())?)
    }
}
