use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use dice_expr::{DiceExpr, DiceExprTerm};
use rand_var::RandomVariable;
use rand_var::rv_traits::{NumRandVar, RandVar};
use rand_var::rv_traits::prob_type::RVProb;

use crate::attack::AtkDmgMap;
use crate::CCError;

pub mod dice_expr;

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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DamageTerm {
    pub expr: DiceExprTerm,
    pub dmg_type: ExtendedDamageType,
}

impl DamageTerm {
    pub fn new(expr: DiceExprTerm, dmg_type: ExtendedDamageType) -> Self {
        DamageTerm {
            expr,
            dmg_type,
        }
    }

    pub fn get_expr(&self) -> &DiceExprTerm {
        &self.expr
    }

    pub fn get_dmg_type(&self) -> &ExtendedDamageType {
        &self.dmg_type
    }
}

pub type DamageExpression<DE> = HashMap<ExtendedDamageType, DE>;

#[derive(Debug, Clone)]
pub struct DamageManager<DE: DiceExpr> {
    pub base_dmg: DamageExpression<DE>,
    pub bonus_crit_dmg: DamageExpression<DE>,
    pub miss_dmg: DamageExpression<DE>,
    damage_features: HashSet<DamageFeature>,
    weapon_die: Option<DamageDice>,
    weapon_dmg_type: Option<DamageType>,
}

impl<DE: DiceExpr + Clone> DamageManager<DE> {
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

    pub fn prebuilt<IDE: Into<DE>>(base: DamageExpression<IDE>, crit: DamageExpression<IDE>, miss: DamageExpression<IDE>) -> Self {
        let mut new_base = HashMap::new();
        for (edt, ide) in base {
            new_base.insert(edt, ide.into());
        }
        let mut new_crit = HashMap::new();
        for (edt, ide) in crit {
            new_crit.insert(edt, ide.into());
        }
        let mut new_miss = HashMap::new();
        for (edt, ide) in miss {
            new_miss.insert(edt, ide.into());
        }
        Self {
            base_dmg: new_base,
            bonus_crit_dmg: new_crit,
            miss_dmg: new_miss,
            damage_features: HashSet::new(),
            weapon_die: None,
            weapon_dmg_type: None,
        }
    }

    pub fn get_dmg_features(&self) -> &HashSet<DamageFeature> {
        &self.damage_features
    }

    pub fn get_weapon_stats(&self) -> Option<(DamageDice, DamageType)> {
        if self.weapon_die.is_none() || self.weapon_dmg_type.is_none() {
            None
        } else {
            Some((self.weapon_die.unwrap(), self.weapon_dmg_type.unwrap()))
        }
    }

    fn merge_dmg(de: &mut DamageExpression<DE>, dtv: Vec<DamageTerm>) {
        for dt in dtv.into_iter() {
            DamageManager::add_dmg_term(de, dt);
        }
    }

    fn add_dmg_term(dmg_expr: &mut DamageExpression<DE>, dmg: DamageTerm) {
        dmg_expr.entry(*dmg.get_dmg_type())
            .and_modify(|de| de.add_term(*dmg.get_expr()))
            .or_insert(DE::from(*dmg.get_expr()));
    }

    pub fn set_weapon(&mut self, die: DamageDice, dmg_type: DamageType) {
        self.weapon_die = Some(die);
        self.weapon_dmg_type = Some(dmg_type);
    }

    pub fn add_base_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.base_dmg, dmg);
    }

    pub fn add_bonus_crit_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.bonus_crit_dmg, dmg);
    }

    pub fn add_miss_dmg(&mut self, dmg: DamageTerm) {
        DamageManager::add_dmg_term(&mut self.miss_dmg, dmg);
    }

    pub fn add_damage_feature(&mut self, dmg_feat: DamageFeature) {
        self.damage_features.insert(dmg_feat);
    }

    pub fn add_all_damage_features(&mut self, new_dmg_feats: HashSet<DamageFeature>) {
        self.damage_features.extend(new_dmg_feats.iter());
    }

    fn get_dmg_type(&self, edt: &ExtendedDamageType) -> Result<DamageType, CCError> {
        match edt {
            ExtendedDamageType::Basic(dt) => Ok(*dt),
            ExtendedDamageType::WeaponDamage => {
                if let Some(dt) = self.weapon_dmg_type {
                    Ok(dt)
                } else {
                    Err(CCError::NoWeaponSet)
                }
            }
        }
    }

    fn get_total_dmg<T: RVProb>(&self, dmg_expr: &DamageExpression<DE>, resistances: &HashSet<DamageType>, double_dice: bool) -> Result<RandomVariable<T>, CCError> {
        let mut rv = RandomVariable::new_constant(0).unwrap();
        for (k, de) in dmg_expr.iter() {
            let mut dice_rv = de.get_dice_rv(&self.damage_features, self.weapon_die)?;
            if double_dice {
                dice_rv = dice_rv.multiple(2);
            }
            dice_rv = dice_rv.add_const(de.get_const());
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

    pub fn get_base_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>, dtv: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        if dtv.len() == 0 {
            self.get_total_dmg(&self.base_dmg, resistances, false)
        } else {
            let mut base_dmg = self.base_dmg.clone();
            DamageManager::merge_dmg(&mut base_dmg, dtv);
            self.get_total_dmg(&base_dmg, resistances, false)
        }
    }

    pub fn get_crit_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>, dtv: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        if dtv.len() == 0 {
            // double base dice + base const
            let mut rv = self.get_total_dmg(&self.base_dmg, resistances, true)?;
            // bonus crit dmg
            rv = rv.add_rv(&self.get_total_dmg(&self.bonus_crit_dmg, resistances, false)?);
            Ok(rv)
        } else {
            let mut base_dmg = self.base_dmg.clone();
            DamageManager::merge_dmg(&mut base_dmg, dtv);
            // double base dice + base const
            let mut rv = self.get_total_dmg(&base_dmg, resistances, true)?;
            // bonus crit dmg
            rv = rv.add_rv(&self.get_total_dmg(&self.bonus_crit_dmg, resistances, false)?);
            Ok(rv)
        }
    }

    pub fn get_miss_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>, dtv: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        if dtv.len() == 0 {
            self.get_total_dmg(&self.miss_dmg, resistances, false)
        } else {
            let mut miss_dmg = self.miss_dmg.clone();
            DamageManager::merge_dmg(&mut miss_dmg, dtv);
            self.get_total_dmg(&miss_dmg, resistances, false)
        }
    }

    // this is often easier for "half dmg on save" than building
    // an actual miss_dmg DamageExpression
    pub fn get_half_base_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CCError> {
        Ok(self.get_base_dmg(resistances, vec!())?.half().unwrap())
    }

    pub fn get_attack_dmg_map<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<AtkDmgMap<T>, CCError> {
        let map = AtkDmgMap::new(
            self.get_miss_dmg(resistances, vec!())?,
            self.get_base_dmg(resistances, vec!())?,
            self.get_crit_dmg(resistances, vec!())?
        );
        Ok(map)
    }
}


#[cfg(test)]
mod tests {
    use rand_var::RV64;
    use crate::damage::dice_expr::DiceExpression;

    use super::*;

    #[test]
    fn test_simple_dmg() {
        let mut dmg: DamageManager<DiceExpression>  = DamageManager::new();
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
        let mut dmg: DamageManager<DiceExpression>  = DamageManager::new();
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
        let mut dmg: DamageManager<DiceExpression> = DamageManager::new();
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
