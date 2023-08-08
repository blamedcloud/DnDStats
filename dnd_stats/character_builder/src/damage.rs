use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Debug, Display};
use num::{FromPrimitive, Integer};
use num::rational::Ratio;
use rand_var::RandomVariable;
use rand_var::rv_traits::{NumRandVar, RandVar};
use crate::attributed_bonus::{AttributedBonus, BonusTerm};
use crate::{CBError, Character};
use crate::combat::attack::AttackResult;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DamageDice {
    D4,
    D6,
    D8,
    D10,
    D12,
    TwoD6,
}

impl DamageDice {
    pub fn get_rv<T>(&self) -> RandomVariable<Ratio<T>>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        match self {
            DamageDice::D4 => RandomVariable::new_dice(4).unwrap(),
            DamageDice::D6 => RandomVariable::new_dice(6).unwrap(),
            DamageDice::D8 => RandomVariable::new_dice(8).unwrap(),
            DamageDice::D10 => RandomVariable::new_dice(10).unwrap(),
            DamageDice::D12 => RandomVariable::new_dice(12).unwrap(),
            DamageDice::TwoD6 => RandomVariable::new_dice(6).unwrap().multiple(2)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

#[derive(Debug, Copy, Clone)]
pub enum DamageInstance {
    Die(DamageDice),
    Dice(u32, DamageDice),
    Const(isize),
}

#[derive(Debug, Copy, Clone)]
pub struct DamageTerm {
    dmg: DamageInstance,
    dmg_type: DamageType,
}

impl DamageTerm {
    pub fn new(dmg: DamageInstance, dmg_type: DamageType) -> Self {
        DamageTerm {
            dmg,
            dmg_type,
        }
    }

    pub fn get_dmg(&self) -> &DamageInstance {
        &self.dmg
    }

    pub fn get_dmg_type(&self) -> &DamageType {
        &self.dmg_type
    }
}

pub struct DamageSum {
    dmg_dice: Vec<DamageDice>,
    dmg_const: isize,
    char_dmg: Option<AttributedBonus>,
}

impl DamageSum {
    pub fn new() -> Self {
        DamageSum {
            dmg_dice: Vec::new(),
            dmg_const: 0,
            char_dmg: None,
        }
    }

    pub fn from(dmg: DamageInstance) -> Self {
        let mut ds = DamageSum::new();
        ds.add_dmg(dmg);
        ds
    }

    pub fn from_char(dmg: BonusTerm) -> Self {
        let mut ds = DamageSum::new();
        ds.add_char_dmg(dmg);
        ds
    }

    pub fn add_dmg(&mut self, dmg: DamageInstance) {
        match dmg {
            DamageInstance::Die(d) => self.dmg_dice.push(d),
            DamageInstance::Dice(num, d) => {
                for _ in 0..num {
                    self.dmg_dice.push(d);
                }
            },
            DamageInstance::Const(c) => self.dmg_const += c,
        };
    }

    pub fn add_char_dmg(&mut self, dmg: BonusTerm) {
        if let None = self.char_dmg {
            self.char_dmg = Some(AttributedBonus::new(String::from("damage const")));
        }
        self.char_dmg.as_mut().unwrap().add_term(dmg);
    }

    pub fn cache_char_dmg(&mut self, character: &Character) {
        self.char_dmg.as_mut().map(|ab| ab.save_value(character));
    }

    pub fn get_cached_char_dmg(&self) -> Result<isize, CBError> {
        if let Some(ab) = &self.char_dmg {
            match ab.get_saved_value() {
                None => Err(CBError::NoCache),
                Some(c) => Ok(c as isize),
            }
        } else {
            Ok(0)
        }
    }

    pub fn get_dmg_const(&self) -> isize {
        self.dmg_const
    }

    pub fn get_total_const(&self) -> Result<isize, CBError> {
        self.get_cached_char_dmg().map(|c| c + self.dmg_const)
    }

    pub fn get_dmg_dice_rv<T>(&self) -> RandomVariable<Ratio<T>>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        for dice in self.dmg_dice.iter() {
            rv = rv.add_rv(&dice.get_rv());
        }
        rv
    }
}

type DamageExpression = HashMap<DamageType, DamageSum>;

pub struct DamageManager {
    base_dmg: DamageExpression,
    bonus_crit_dmg: DamageExpression,
    miss_dmg: DamageExpression,
}

impl DamageManager {
    pub fn new() -> Self {
        DamageManager {
            base_dmg: HashMap::new(),
            bonus_crit_dmg: HashMap::new(),
            miss_dmg: HashMap::new(),
        }
    }

    fn add_dmg_term(de: &mut DamageExpression, dmg: DamageTerm) {
        de.entry(*dmg.get_dmg_type())
            .and_modify(|ds| ds.add_dmg(*dmg.get_dmg()))
            .or_insert(DamageSum::from(*dmg.get_dmg()));
    }

    fn add_char_dmg_term(de: &mut DamageExpression, dmg_type: DamageType, dmg: BonusTerm) {
        // can't use the fancy entry API because BonusTerm isn't cloneable
        // and (as of this writing), rustc isn't smart enough to figure out
        // that only one of and_modify or or_insert will be called.
        if de.contains_key(&dmg_type) {
            de.get_mut(&dmg_type).unwrap().add_char_dmg(dmg);
        } else {
            de.insert(dmg_type, DamageSum::from_char(dmg));
        }
    }

    pub fn add_base_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.base_dmg, dmg);
    }
    pub fn add_base_char_dmg(&mut self, dmg_type: DamageType, dmg: BonusTerm) {
        DamageManager::add_char_dmg_term(&mut self.base_dmg, dmg_type, dmg);
    }

    pub fn add_bonus_crit_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.bonus_crit_dmg, dmg);
    }
    pub fn add_crit_char_dmg(&mut self, dmg_type: DamageType, dmg: BonusTerm) {
        DamageManager::add_char_dmg_term(&mut self.bonus_crit_dmg, dmg_type, dmg);
    }

    pub fn add_miss_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.miss_dmg, dmg);
    }
    pub fn add_miss_char_dmg(&mut self, dmg_type: DamageType, dmg: BonusTerm) {
        DamageManager::add_char_dmg_term(&mut self.miss_dmg, dmg_type, dmg);
    }

    pub fn cache_char_dmg(&mut self, character: &Character) {
        for (_, ds) in self.base_dmg.iter_mut() {
            ds.cache_char_dmg(character);
        }
        for (_, ds) in self.bonus_crit_dmg.iter_mut() {
            ds.cache_char_dmg(character);
        }
        for (_, ds) in self.miss_dmg.iter_mut() {
            ds.cache_char_dmg(character);
        }
    }

    fn get_total_dmg<T>(de: &DamageExpression, resistances: &HashSet<DamageType>, double_dice: bool) -> Result<RandomVariable<Ratio<T>>, CBError>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        for (k, ds) in de.iter() {
            let type_rv;
            if double_dice {
                type_rv = ds.get_dmg_dice_rv().multiple(2);
            } else {
                type_rv = ds.get_dmg_dice_rv();
            }
            let type_rv = type_rv.add_const(ds.get_total_const()?);
            if resistances.contains(k) {
                rv = rv.add_rv(&type_rv.half().unwrap());
            } else {
                rv = rv.add_rv(&type_rv);
            }
        }
        // damage is never negative
        rv = rv.cap_lb(0).unwrap();
        Ok(rv)
    }

    pub fn get_base_dmg<T>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<Ratio<T>>, CBError>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        DamageManager::get_total_dmg(&self.base_dmg, resistances, false)
    }

    pub fn get_crit_dmg<T>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<Ratio<T>>, CBError>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        // double base dice + base const
        let mut rv = DamageManager::get_total_dmg(&self.base_dmg, resistances, true)?;
        // bonus crit dmg
        rv = rv.add_rv(&DamageManager::get_total_dmg(&self.bonus_crit_dmg, resistances, false)?);
        Ok(rv)
    }

    pub fn get_miss_dmg<T>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<Ratio<T>>, CBError>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        DamageManager::get_total_dmg(&self.miss_dmg, resistances, false)
    }

    // this is often easier for "half dmg on save" than building
    // an actual miss_dmg DamageExpression
    pub fn get_half_base_dmg<T>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<Ratio<T>>, CBError>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        Ok(self.get_base_dmg(resistances)?.half().unwrap())
    }

    pub fn get_attack_dmg_map<T>(&self, resistances: &HashSet<DamageType>) -> Result<BTreeMap<AttackResult, RandomVariable<Ratio<T>>>, CBError>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        let mut map = BTreeMap::new();
        map.insert(AttackResult::Miss, self.get_miss_dmg(resistances)?);
        map.insert(AttackResult::Hit, self.get_base_dmg(resistances)?);
        map.insert(AttackResult::Crit, self.get_crit_dmg(resistances)?);
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use num::Rational64;
    use super::*;

    #[test]
    fn test_simple_dmg() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Die(DamageDice::D6), DamageType::Bludgeoning));
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Const(3), DamageType::Bludgeoning));
        let rv1: RandomVariable<Rational64> = dmg.get_base_dmg(&HashSet::new()).unwrap();

        let rv2: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap();
        let rv2 = rv2.add_const(3);
        assert_eq!(rv1, rv2);

        let rv3: RandomVariable<Rational64> = dmg.get_crit_dmg(&HashSet::new()).unwrap();

        let rv4: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(2);
        let rv4 = rv4.add_const(3);
        assert_eq!(rv3, rv4);
    }

    #[test]
    fn test_brutal_crit() {
        let mut dmg = DamageManager::new();
        let d12_sl = DamageTerm::new(DamageInstance::Die(DamageDice::D12), DamageType::Slashing);
        dmg.add_base_dmg(d12_sl);
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Const(5), DamageType::Slashing));
        dmg.add_bonus_crit_dmg(d12_sl);
        let rv1: RandomVariable<Rational64> = dmg.get_base_dmg(&HashSet::new()).unwrap();

        let d12: RandomVariable<Rational64> = RandomVariable::new_dice(12).unwrap();
        let const_dmg = 5;
        let base_dmg = d12.add_const(const_dmg);
        assert_eq!(rv1, base_dmg);

        let rv2: RandomVariable<Rational64> = dmg.get_crit_dmg(&HashSet::new()).unwrap();
        let crit_dmg = d12.multiple(3).add_const(const_dmg);
        assert_eq!(rv2, crit_dmg);
    }

    #[test]
    fn test_flame_strike() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Dice(4,DamageDice::D6), DamageType::Fire));
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Dice(4,DamageDice::D6), DamageType::Radiant));
        let rv1: RandomVariable<Rational64> = dmg.get_base_dmg(&HashSet::new()).unwrap();
        let rv2: RandomVariable<Rational64> = dmg.get_half_base_dmg(&HashSet::new()).unwrap();

        let eight_d6: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(8);
        assert_eq!(rv1, eight_d6);
        let save_dmg = eight_d6.half().unwrap();
        assert_eq!(rv2, save_dmg);

        let resist_fire = HashSet::from([DamageType::Fire]);

        let rv3: RandomVariable<Rational64> = dmg.get_base_dmg(&resist_fire).unwrap();
        let four_d6: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(4);
        let resist_dmg = four_d6.add_rv(&four_d6.half().unwrap());
        assert_eq!(rv3, resist_dmg);

        let rv4: RandomVariable<Rational64> = dmg.get_half_base_dmg(&resist_fire).unwrap();
        let resist_save_dmg = resist_dmg.half().unwrap();
        assert_eq!(rv4, resist_save_dmg);
    }

}