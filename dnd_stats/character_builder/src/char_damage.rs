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
pub struct CharDmgManager { // TODO: this implementation is a hack. re-implement this better ...
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

impl From<CharDmgManager> for DamageManager<DiceExpression> {
    fn from(value: CharDmgManager) -> Self {
        let dmg_feats = value.cdm.get_dmg_features().clone();
        let weapon = value.cdm.get_weapon_stats();
        let mut dmg_manager = DamageManager::prebuilt(value.cdm.base_dmg, value.cdm.bonus_crit_dmg, value.cdm.miss_dmg);
        dmg_manager.add_all_damage_features(dmg_feats);
        if weapon.is_some() {
            dmg_manager.set_weapon(weapon.unwrap().0, weapon.unwrap().1);
        }
        dmg_manager
    }
}
