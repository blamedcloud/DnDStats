use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use rand_var::RandomVariable;
use rand_var::rv_traits::{NumRandVar, RandVar};
use rand_var::rv_traits::prob_type::RVProb;
use crate::attributed_bonus::{AttributedBonus, BonusTerm};
use crate::{CBError, Character};
use crate::combat::attack::AttackResult;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DamageDice {
    D4,
    D6,
    D8,
    D10,
    D12,
    TwoD6,
}

impl DamageDice {
    pub fn get_rv<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            DamageDice::D4 => RandomVariable::new_dice(4).unwrap(),
            DamageDice::D6 => RandomVariable::new_dice(6).unwrap(),
            DamageDice::D8 => RandomVariable::new_dice(8).unwrap(),
            DamageDice::D10 => RandomVariable::new_dice(10).unwrap(),
            DamageDice::D12 => RandomVariable::new_dice(12).unwrap(),
            DamageDice::TwoD6 => RandomVariable::new_dice(6).unwrap().multiple(2)
        }
    }

    pub fn get_rv_gwf<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            DamageDice::D4 => RandomVariable::new_dice_reroll(4, 2).unwrap(),
            DamageDice::D6 => RandomVariable::new_dice_reroll(6, 2).unwrap(),
            DamageDice::D8 => RandomVariable::new_dice_reroll(8, 2).unwrap(),
            DamageDice::D10 => RandomVariable::new_dice_reroll(10, 2).unwrap(),
            DamageDice::D12 => RandomVariable::new_dice_reroll(12, 2).unwrap(),
            DamageDice::TwoD6 => RandomVariable::new_dice_reroll(6, 2).unwrap().multiple(2)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ExtendedDamageDice {
    Basic(DamageDice),
    WeaponDice,
    SingleWeaponDie, // used for brutal critical for example
}

impl ExtendedDamageDice {
    pub fn get_single_die(dd: DamageDice) -> DamageDice {
        match dd {
            DamageDice::TwoD6 => DamageDice::D6,
            d => d,
        }
    }
}

impl From<DamageDice> for ExtendedDamageDice {
    fn from(value: DamageDice) -> Self {
        ExtendedDamageDice::Basic(value)
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum DamageFeature {
    GWF,
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

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum ExtendedDamageType {
    Basic(DamageType),
    WeaponDamage,
}

impl From<DamageType> for ExtendedDamageType {
    fn from(value: DamageType) -> Self {
        ExtendedDamageType::Basic(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DamageInstance {
    Die(ExtendedDamageDice),
    Dice(u32, ExtendedDamageDice),
    Const(isize),
}

#[derive(Debug, Copy, Clone)]
pub struct DamageTerm {
    dmg: DamageInstance,
    dmg_type: ExtendedDamageType,
}

impl DamageTerm {
    pub fn new(dmg: DamageInstance, dmg_type: ExtendedDamageType) -> Self {
        DamageTerm {
            dmg,
            dmg_type,
        }
    }

    pub fn get_dmg(&self) -> &DamageInstance {
        &self.dmg
    }

    pub fn get_dmg_type(&self) -> &ExtendedDamageType {
        &self.dmg_type
    }
}

#[derive(Clone)]
pub struct DamageSum {
    dmg_dice: Vec<ExtendedDamageDice>,
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

    fn get_die(ext_dice: &ExtendedDamageDice, weapon_dmg: Option<DamageDice>) -> Result<DamageDice, CBError> {
        match ext_dice {
            ExtendedDamageDice::Basic(d) => Ok(*d),
            ExtendedDamageDice::WeaponDice => {
                if let Some(d) = weapon_dmg {
                    Ok(d)
                } else {
                    Err(CBError::NoWeaponSet)
                }
            },
            ExtendedDamageDice::SingleWeaponDie => {
                if let Some(d) = weapon_dmg {
                    Ok(ExtendedDamageDice::get_single_die(d))
                } else {
                    Err(CBError::NoWeaponSet)
                }
            },
        }
    }

    pub fn get_dmg_dice_rv<T: RVProb>(&self, dmg_feats: &HashSet<DamageFeature>, weapon_dmg: Option<DamageDice>) -> Result<RandomVariable<T>, CBError> {
        let gwf = dmg_feats.contains(&DamageFeature::GWF);
        let mut rv: RandomVariable<T> = RandomVariable::new_constant(0).unwrap();
        for ext_dice in self.dmg_dice.iter() {
            let dice = DamageSum::get_die(ext_dice, weapon_dmg)?;
            if gwf {
                rv = rv.add_rv(&dice.get_rv_gwf());
            } else {
                rv = rv.add_rv(&dice.get_rv());
            }
        }
        Ok(rv)
    }
}

type DamageExpression = HashMap<ExtendedDamageType, DamageSum>;

#[derive(Clone)]
pub struct DamageManager {
    base_dmg: DamageExpression,
    bonus_crit_dmg: DamageExpression,
    miss_dmg: DamageExpression,
    damage_features: HashSet<DamageFeature>,
    weapon_die: Option<DamageDice>,
    weapon_dmg_type: Option<DamageType>,
}

impl DamageManager {
    pub fn new() -> Self {
        DamageManager {
            base_dmg: HashMap::new(),
            bonus_crit_dmg: HashMap::new(),
            miss_dmg: HashMap::new(),
            damage_features: HashSet::new(),
            weapon_die: None,
            weapon_dmg_type: None,
        }
    }

    fn add_dmg_term(de: &mut DamageExpression, dmg: DamageTerm) {
        de.entry(*dmg.get_dmg_type())
            .and_modify(|ds| ds.add_dmg(*dmg.get_dmg()))
            .or_insert(DamageSum::from(*dmg.get_dmg()));
    }

    fn add_char_dmg_term(de: &mut DamageExpression, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        if de.contains_key(&dmg_type) {
            de.get_mut(&dmg_type).unwrap().add_char_dmg(dmg);
        } else {
            de.insert(dmg_type, DamageSum::from_char(dmg));
        }
    }

    pub fn set_weapon(&mut self, die: DamageDice, dmg_type: DamageType) {
        self.weapon_die = Some(die);
        self.weapon_dmg_type = Some(dmg_type);
    }

    pub fn add_base_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.base_dmg, dmg);
    }
    pub fn add_base_char_dmg(&mut self, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        DamageManager::add_char_dmg_term(&mut self.base_dmg, dmg_type, dmg);
    }

    pub fn add_bonus_crit_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.bonus_crit_dmg, dmg);
    }
    pub fn add_crit_char_dmg(&mut self, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        DamageManager::add_char_dmg_term(&mut self.bonus_crit_dmg, dmg_type, dmg);
    }

    pub fn add_miss_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.miss_dmg, dmg);
    }
    pub fn add_miss_char_dmg(&mut self, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        DamageManager::add_char_dmg_term(&mut self.miss_dmg, dmg_type, dmg);
    }

    pub fn add_damage_feature(&mut self, dmg_feat: DamageFeature) {
        self.damage_features.insert(dmg_feat);
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

    fn get_dmg_type(&self, edt: &ExtendedDamageType) -> Result<DamageType, CBError> {
        match edt {
            ExtendedDamageType::Basic(dt) => Ok(*dt),
            ExtendedDamageType::WeaponDamage => {
                if let Some(dt) = self.weapon_dmg_type {
                    Ok(dt)
                } else {
                    Err(CBError::NoWeaponSet)
                }
            }
        }
    }

    fn get_total_dmg<T: RVProb>(&self, de: &DamageExpression, resistances: &HashSet<DamageType>, double_dice: bool) -> Result<RandomVariable<T>, CBError> {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        for (k, ds) in de.iter() {
            let mut dice_rv = ds.get_dmg_dice_rv(&self.damage_features, self.weapon_die)?;
            if double_dice {
                dice_rv = dice_rv.multiple(2);
            }
            dice_rv = dice_rv.add_const(ds.get_total_const()?);
            let dmg_type = self.get_dmg_type(k)?;
            if resistances.contains(&dmg_type) {
                rv = rv.add_rv(&dice_rv.half().unwrap());
            } else {
                rv = rv.add_rv(&dice_rv);
            }
        }
        // damage is never negative
        rv = rv.cap_lb(0).unwrap();
        Ok(rv)
    }

    pub fn get_base_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        self.get_total_dmg(&self.base_dmg, resistances, false)
    }

    pub fn get_crit_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        // double base dice + base const
        let mut rv = self.get_total_dmg(&self.base_dmg, resistances, true)?;
        // bonus crit dmg
        rv = rv.add_rv(&self.get_total_dmg(&self.bonus_crit_dmg, resistances, false)?);
        Ok(rv)
    }

    pub fn get_miss_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        self.get_total_dmg(&self.miss_dmg, resistances, false)
    }

    // this is often easier for "half dmg on save" than building
    // an actual miss_dmg DamageExpression
    pub fn get_half_base_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        Ok(self.get_base_dmg(resistances)?.half().unwrap())
    }

    pub fn get_attack_dmg_map<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<BTreeMap<AttackResult, RandomVariable<T>>, CBError> {
        let mut map = BTreeMap::new();
        map.insert(AttackResult::Miss, self.get_miss_dmg(resistances)?);
        map.insert(AttackResult::Hit, self.get_base_dmg(resistances)?);
        map.insert(AttackResult::Crit, self.get_crit_dmg(resistances)?);
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use rand_var::RV64;
    use super::*;

    #[test]
    fn test_simple_dmg() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Die(ExtendedDamageDice::Basic(DamageDice::D6)), ExtendedDamageType::Basic(DamageType::Bludgeoning)));
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Const(3), ExtendedDamageType::Basic(DamageType::Bludgeoning)));
        let rv1: RV64 = dmg.get_base_dmg(&HashSet::new()).unwrap();

        let rv2: RV64 = RandomVariable::new_dice(6).unwrap();
        let rv2 = rv2.add_const(3);
        assert_eq!(rv1, rv2);

        let rv3: RV64 = dmg.get_crit_dmg(&HashSet::new()).unwrap();

        let rv4: RV64 = RandomVariable::new_dice(6).unwrap().multiple(2);
        let rv4 = rv4.add_const(3);
        assert_eq!(rv3, rv4);
    }

    #[test]
    fn test_brutal_crit() {
        let mut dmg = DamageManager::new();
        dmg.set_weapon(DamageDice::D12, DamageType::Slashing);
        let weapon_dmg = DamageTerm::new(DamageInstance::Die(ExtendedDamageDice::WeaponDice), ExtendedDamageType::WeaponDamage);
        dmg.add_base_dmg(weapon_dmg);
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Const(5), ExtendedDamageType::WeaponDamage));
        let brutal_crit_dmg = DamageTerm::new(DamageInstance::Die(ExtendedDamageDice::SingleWeaponDie), ExtendedDamageType::WeaponDamage);
        dmg.add_bonus_crit_dmg(brutal_crit_dmg);
        let rv1: RV64 = dmg.get_base_dmg(&HashSet::new()).unwrap();

        let d12: RV64 = RandomVariable::new_dice(12).unwrap();
        let const_dmg = 5;
        let base_dmg = d12.add_const(const_dmg);
        assert_eq!(rv1, base_dmg);

        let rv2: RV64 = dmg.get_crit_dmg(&HashSet::new()).unwrap();
        let crit_dmg = d12.multiple(3).add_const(const_dmg);
        assert_eq!(rv2, crit_dmg);
    }

    #[test]
    fn test_flame_strike() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Dice(4,DamageDice::D6.into()), DamageType::Fire.into()));
        dmg.add_base_dmg(DamageTerm::new(DamageInstance::Dice(4,DamageDice::D6.into()), DamageType::Radiant.into()));
        let rv1: RV64 = dmg.get_base_dmg(&HashSet::new()).unwrap();
        let rv2: RV64 = dmg.get_half_base_dmg(&HashSet::new()).unwrap();

        let eight_d6: RV64 = RandomVariable::new_dice(6).unwrap().multiple(8);
        assert_eq!(rv1, eight_d6);
        let save_dmg = eight_d6.half().unwrap();
        assert_eq!(rv2, save_dmg);

        let resist_fire = HashSet::from([DamageType::Fire]);

        let rv3: RV64 = dmg.get_base_dmg(&resist_fire).unwrap();
        let four_d6: RV64 = RandomVariable::new_dice(6).unwrap().multiple(4);
        let resist_dmg = four_d6.add_rv(&four_d6.half().unwrap());
        assert_eq!(rv3, resist_dmg);

        let rv4: RV64 = dmg.get_half_base_dmg(&resist_fire).unwrap();
        let resist_save_dmg = resist_dmg.half().unwrap();
        assert_eq!(rv4, resist_save_dmg);
    }

}