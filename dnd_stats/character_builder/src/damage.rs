use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use num::{FromPrimitive, Integer};
use num::rational::Ratio;
use rand_var::RandomVariable;
use rand_var::rv_traits::NumRandVar;

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
}

impl DamageSum {
    pub fn new() -> Self {
        DamageSum {
            dmg_dice: Vec::new(),
            dmg_const: 0,
        }
    }

    pub fn from(dmg: DamageInstance) -> Self {
        let mut ds = DamageSum::new();
        ds.add_dmg(dmg);
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

    pub fn get_dmg_const(&self) -> isize {
        self.dmg_const
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

    pub fn add_base_dmg(&mut self, dmg: DamageTerm) {
        self.base_dmg.entry(*dmg.get_dmg_type())
            .and_modify(|ds| ds.add_dmg(*dmg.get_dmg()))
            .or_insert(DamageSum::from(*dmg.get_dmg()));
    }

    pub fn add_bonus_crit_dmg(&mut self, dmg: DamageTerm) {
        self.bonus_crit_dmg.entry(*dmg.get_dmg_type())
            .and_modify(|ds| ds.add_dmg(*dmg.get_dmg()))
            .or_insert(DamageSum::from(*dmg.get_dmg()));
    }

    pub fn add_miss_dmg(&mut self, dmg: DamageTerm) {
        self.miss_dmg.entry(*dmg.get_dmg_type())
            .and_modify(|ds| ds.add_dmg(*dmg.get_dmg()))
            .or_insert(DamageSum::from(*dmg.get_dmg()));
    }

    pub fn get_base_dmg<T>(&self, resistances: &HashSet<DamageType>) -> RandomVariable<Ratio<T>>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        for (k, ds) in self.base_dmg.iter() {
            let type_rv = ds.get_dmg_dice_rv();
            let const_rv = RandomVariable::new_constant(ds.get_dmg_const()).unwrap();
            let type_rv = type_rv.add_rv(&const_rv);
            if resistances.contains(k) {
                rv = rv.add_rv(&type_rv.half().unwrap());
            } else {
                rv = rv.add_rv(&type_rv);
            }
        }
        rv
    }

    pub fn get_crit_dmg<T>(&self, resistances: &HashSet<DamageType>) -> RandomVariable<Ratio<T>>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        // double base dice + base const
        for (k, ds) in self.base_dmg.iter() {
            let type_rv = ds.get_dmg_dice_rv().multiple(2);
            let const_rv = RandomVariable::new_constant(ds.get_dmg_const()).unwrap();
            let type_rv = type_rv.add_rv(&const_rv);
            if resistances.contains(k) {
                rv = rv.add_rv(&type_rv.half().unwrap());
            } else {
                rv = rv.add_rv(&type_rv);
            }
        }
        // bonus crit dmg
        for (k, ds) in self.bonus_crit_dmg.iter() {
            let type_rv = ds.get_dmg_dice_rv();
            let const_rv = RandomVariable::new_constant(ds.get_dmg_const()).unwrap();
            let type_rv = type_rv.add_rv(&const_rv);
            if resistances.contains(k) {
                rv = rv.add_rv(&type_rv.half().unwrap());
            } else {
                rv = rv.add_rv(&type_rv);
            }
        }
        rv
    }

    pub fn get_miss_dmg<T>(&self, resistances: &HashSet<DamageType>) -> RandomVariable<Ratio<T>>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        for (k, ds) in self.miss_dmg.iter() {
            let type_rv = ds.get_dmg_dice_rv();
            let const_rv = RandomVariable::new_constant(ds.get_dmg_const()).unwrap();
            let type_rv = type_rv.add_rv(&const_rv);
            if resistances.contains(k) {
                rv = rv.add_rv(&type_rv.half().unwrap());
            } else {
                rv = rv.add_rv(&type_rv);
            }
        }
        rv
    }

    // this is often easier for "half dmg on save" than building
    // an actual miss_dmg DamageExpression
    pub fn get_half_base_dmg<T>(&self, resistances: &HashSet<DamageType>) -> RandomVariable<Ratio<T>>
    where
        T: Integer + Debug + Clone + Display + FromPrimitive,
        Ratio<T>: FromPrimitive
    {
        self.get_base_dmg(resistances).half().unwrap()
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
        let rv1: RandomVariable<Rational64> = dmg.get_base_dmg(&HashSet::new());

        let rv2: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap();
        let rv2 = rv2.add_rv(&RandomVariable::new_constant(3).unwrap());
        assert_eq!(rv1, rv2);

        let rv3: RandomVariable<Rational64> = dmg.get_crit_dmg(&HashSet::new());

        let rv4: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(2);
        let rv4 = rv4.add_rv(&RandomVariable::new_constant(3).unwrap());
        assert_eq!(rv3, rv4);
    }

    #[test]
    fn test_brutal_crit() {
        let mut dmg = DamageManager::new();
        let d12_sl = DamageTerm::new(DamageInstance::Die(DamageDice::D12), DamageType::Slashing);
        dmg.add_base_dmg(d12_sl);
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Const(5), DamageType::Slashing));
        dmg.add_bonus_crit_dmg(d12_sl);
        let rv1: RandomVariable<Rational64> = dmg.get_base_dmg(&HashSet::new());

        let d12: RandomVariable<Rational64> = RandomVariable::new_dice(12).unwrap();
        let const_rv: RandomVariable<Rational64> = RandomVariable::new_constant(5).unwrap();
        let base_dmg = d12.add_rv(&const_rv);
        assert_eq!(rv1, base_dmg);

        let rv2: RandomVariable<Rational64> = dmg.get_crit_dmg(&HashSet::new());
        let crit_dmg = d12.multiple(3).add_rv(&const_rv);
        assert_eq!(rv2, crit_dmg);
    }

    #[test]
    fn test_flame_strike() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Dice(4,DamageDice::D6), DamageType::Fire));
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Dice(4,DamageDice::D6), DamageType::Radiant));
        let rv1: RandomVariable<Rational64> = dmg.get_base_dmg(&HashSet::new());
        let rv2: RandomVariable<Rational64> = dmg.get_half_base_dmg(&HashSet::new());

        let eight_d6: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(8);
        assert_eq!(rv1, eight_d6);
        let save_dmg = eight_d6.half().unwrap();
        assert_eq!(rv2, save_dmg);

        let resist_fire = HashSet::from([DamageType::Fire]);

        let rv3: RandomVariable<Rational64> = dmg.get_base_dmg(&resist_fire);
        let four_d6: RandomVariable<Rational64> = RandomVariable::new_dice(6).unwrap().multiple(4);
        let resist_dmg = four_d6.add_rv(&four_d6.half().unwrap());
        assert_eq!(rv3, resist_dmg);

        let rv4: RandomVariable<Rational64> = dmg.get_half_base_dmg(&resist_fire);
        let resist_save_dmg = resist_dmg.half().unwrap();
        assert_eq!(rv4, resist_save_dmg);
    }

}