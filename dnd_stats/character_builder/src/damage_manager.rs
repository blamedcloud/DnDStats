use std::collections::HashSet;
use std::fmt::Debug;

use combat_core::CCError;
use combat_core::damage::{DamageDice, DamageExpression, DamageFeature, DamageManager, ExtendedDamageDice, ExtendedDamageType};
use combat_core::damage::dice_expr::{DiceExpr, DiceExpression, DiceExprTerm};
use rand_var::RandomVariable;
use rand_var::rv_traits::{NumRandVar};
use rand_var::rv_traits::prob_type::RVProb;

use crate::{CBError, Character};
use crate::attributed_bonus::{AttributedBonus, BonusTerm};

#[derive(Debug, Clone)]
pub struct CharDiceExpr {
    dice_terms: Vec<ExtendedDamageDice>,
    const_term: isize,
    char_terms: Option<AttributedBonus>,
}

impl CharDiceExpr {
    pub fn new() -> Self {
        CharDiceExpr {
            dice_terms: Vec::new(),
            const_term: 0,
            char_terms: None,
        }
    }

    pub fn add_char_term(&mut self, term: BonusTerm) {
        if let None = self.char_terms {
            self.char_terms = Some(AttributedBonus::new(String::from("dice expr const")));
        }
        self.char_terms.as_mut().unwrap().add_term(term);
    }

    pub fn cache_char_terms(&mut self, character: &Character) {
        self.char_terms.as_mut().map(|ab| ab.save_value(character));
    }

    pub fn get_cached_char_terms(&self) -> Result<isize, CBError> {
        if let Some(ab) = &self.char_terms {
            match ab.get_saved_value() {
                None => Err(CBError::NoCache),
                Some(c) => Ok(c as isize),
            }
        } else {
            Ok(0)
        }
    }

    pub fn get_const_term(&self) -> isize {
        self.const_term
    }

    pub fn get_total_const(&self) -> Result<isize, CBError> {
        self.get_cached_char_terms().map(|c| c + self.const_term)
    }

    fn get_die(ext_dice: &ExtendedDamageDice, weapon_die: Option<DamageDice>) -> Result<DamageDice, CBError> {
        match ext_dice {
            ExtendedDamageDice::Basic(d) => Ok(*d),
            ExtendedDamageDice::WeaponDice => {
                if let Some(d) = weapon_die {
                    Ok(d)
                } else {
                    Err(CBError::NoWeaponSet)
                }
            },
            ExtendedDamageDice::SingleWeaponDie => {
                if let Some(d) = weapon_die {
                    Ok(ExtendedDamageDice::get_single_die(d))
                } else {
                    Err(CBError::NoWeaponSet)
                }
            },
        }
    }
}

impl From<CharDiceExpr> for DiceExpression {
    fn from(value: CharDiceExpr) -> Self {
        let const_term = value.get_total_const().unwrap_or(value.const_term);
        (value.dice_terms, const_term).into()
    }
}

impl From<DiceExprTerm> for CharDiceExpr {
    fn from(value: DiceExprTerm) -> Self {
        let mut ds = CharDiceExpr::new();
        ds.add_term(value);
        ds
    }
}
impl From<BonusTerm> for CharDiceExpr {
    fn from(value: BonusTerm) -> Self {
        let mut ds = CharDiceExpr::new();
        ds.add_char_term(value);
        ds
    }
}

impl DiceExpr for CharDiceExpr {
    fn add_term(&mut self, term: DiceExprTerm) {
        match term {
            DiceExprTerm::Die(d) => self.dice_terms.push(d),
            DiceExprTerm::Dice(num, d) => {
                for _ in 0..num {
                    self.dice_terms.push(d);
                }
            },
            DiceExprTerm::Const(c) => self.const_term += c,
        };
    }

    fn get_dice_rv<T: RVProb>(&self, dmg_feats: &HashSet<DamageFeature>, weapon_dmg: Option<DamageDice>) -> Result<RandomVariable<T>, CCError> {
        let gwf = dmg_feats.contains(&DamageFeature::GWF);
        let mut rv: RandomVariable<T> = RandomVariable::new_constant(0).unwrap();
        for ext_dice in self.dice_terms.iter() {
            let dice = CharDiceExpr::get_die(ext_dice, weapon_dmg)?;
            if gwf {
                rv = rv.add_rv(&dice.get_rv_gwf());
            } else {
                rv = rv.add_rv(&dice.get_rv());
            }
        }
        Ok(rv)
    }

    fn get_const(&self) -> isize {
        self.get_total_const().unwrap_or(self.const_term)
    }
}

#[derive(Debug, Clone)]
pub struct CharDmgManager {
    pub cdm: DamageManager<CharDiceExpr>,
}

impl CharDmgManager {
    pub fn new() -> Self {
        Self {
            cdm: DamageManager::new(),
        }
    }

    fn add_char_dmg_term(de: &mut DamageExpression<CharDiceExpr>, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        if de.contains_key(&dmg_type) {
            de.get_mut(&dmg_type).unwrap().add_char_term(dmg);
        } else {
            de.insert(dmg_type, dmg.into());
        }
    }

    pub fn add_base_char_dmg(&mut self, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        CharDmgManager::add_char_dmg_term(&mut self.cdm.base_dmg, dmg_type, dmg);
    }

    pub fn add_crit_char_dmg(&mut self, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        CharDmgManager::add_char_dmg_term(&mut self.cdm.bonus_crit_dmg, dmg_type, dmg);
    }

    pub fn add_miss_char_dmg(&mut self, dmg_type: ExtendedDamageType, dmg: BonusTerm) {
        CharDmgManager::add_char_dmg_term(&mut self.cdm.miss_dmg, dmg_type, dmg);
    }

    pub fn cache_char_dmg(&mut self, character: &Character) {
        for (_, ds) in self.cdm.base_dmg.iter_mut() {
            ds.cache_char_terms(character);
        }
        for (_, ds) in self.cdm.bonus_crit_dmg.iter_mut() {
            ds.cache_char_terms(character);
        }
        for (_, ds) in self.cdm.miss_dmg.iter_mut() {
            ds.cache_char_terms(character);
        }
    }
}


#[cfg(test)]
mod tests {
    use combat_core::damage::DamageManager;
    use rand_var::RV64;

    use super::*;

    #[test]
    fn test_simple_dmg() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DiceExprTerm::Die(ExtendedDamageDice::Basic(DamageDice::D6)), ExtendedDamageType::Basic(DamageType::Bludgeoning)));
        dmg.add_base_dmg(DamageTerm::new(DiceExprTerm::Const(3), ExtendedDamageType::Basic(DamageType::Bludgeoning)));
        let rv1: RV64 = dmg.get_base_dmg(&HashSet::new(), vec!()).unwrap();

        let rv2: RV64 = RandomVariable::new_dice(6).unwrap();
        let rv2 = rv2.add_const(3);
        assert_eq!(rv1, rv2);

        let rv3: RV64 = dmg.get_crit_dmg(&HashSet::new(), vec!()).unwrap();

        let rv4: RV64 = RandomVariable::new_dice(6).unwrap().multiple(2);
        let rv4 = rv4.add_const(3);
        assert_eq!(rv3, rv4);
    }

    #[test]
    fn test_brutal_crit() {
        let mut dmg = DamageManager::new();
        dmg.set_weapon(DamageDice::D12, DamageType::Slashing);
        let weapon_dmg = DamageTerm::new(DiceExprTerm::Die(ExtendedDamageDice::WeaponDice), ExtendedDamageType::WeaponDamage);
        dmg.add_base_dmg(weapon_dmg);
        dmg.add_base_dmg(DamageTerm::new(DiceExprTerm::Const(5), ExtendedDamageType::WeaponDamage));
        let brutal_crit_dmg = DamageTerm::new(DiceExprTerm::Die(ExtendedDamageDice::SingleWeaponDie), ExtendedDamageType::WeaponDamage);
        dmg.add_bonus_crit_dmg(brutal_crit_dmg);
        let rv1: RV64 = dmg.get_base_dmg(&HashSet::new(), vec!()).unwrap();

        let d12: RV64 = RandomVariable::new_dice(12).unwrap();
        let const_dmg = 5;
        let base_dmg = d12.add_const(const_dmg);
        assert_eq!(rv1, base_dmg);

        let rv2: RV64 = dmg.get_crit_dmg(&HashSet::new(), vec!()).unwrap();
        let crit_dmg = d12.multiple(3).add_const(const_dmg);
        assert_eq!(rv2, crit_dmg);
    }

    #[test]
    fn test_flame_strike() {
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(DiceExprTerm::Dice(4, DamageDice::D6.into()), DamageType::Fire.into()));
        dmg.add_base_dmg(DamageTerm::new(DiceExprTerm::Dice(4, DamageDice::D6.into()), DamageType::Radiant.into()));
        let rv1: RV64 = dmg.get_base_dmg(&HashSet::new(), vec!()).unwrap();
        let rv2: RV64 = dmg.get_half_base_dmg(&HashSet::new()).unwrap();

        let eight_d6: RV64 = RandomVariable::new_dice(6).unwrap().multiple(8);
        assert_eq!(rv1, eight_d6);
        let save_dmg = eight_d6.half().unwrap();
        assert_eq!(rv2, save_dmg);

        let resist_fire = HashSet::from([DamageType::Fire]);

        let rv3: RV64 = dmg.get_base_dmg(&resist_fire, vec!()).unwrap();
        let four_d6: RV64 = RandomVariable::new_dice(6).unwrap().multiple(4);
        let resist_dmg = four_d6.add_rv(&four_d6.half().unwrap());
        assert_eq!(rv3, resist_dmg);

        let rv4: RV64 = dmg.get_half_base_dmg(&resist_fire).unwrap();
        let resist_save_dmg = resist_dmg.half().unwrap();
        assert_eq!(rv4, resist_save_dmg);
    }

}