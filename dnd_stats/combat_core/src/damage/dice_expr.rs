use std::collections::HashSet;
use std::fmt::Debug;

use rand_var::vec_rand_var::VecRandVar;
use rand_var::num_rand_var::NumRandVar;
use rand_var::rand_var::prob_type::RVProb;

use crate::CCError;
use crate::damage::{DamageDice, DamageFeature, ExtendedDamageDice};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DiceExprTerm {
    Die(ExtendedDamageDice),
    Dice(u8, ExtendedDamageDice),
    Const(isize),
}

#[derive(Debug, Clone)]
pub struct DiceExpression {
    dice_terms: Vec<ExtendedDamageDice>,
    const_term: isize,
}

impl DiceExpression {
    pub fn new() -> Self {
        Self {
            dice_terms: Vec::new(),
            const_term: 0,
        }
    }

    pub fn get_const_term(&self) -> isize {
        self.const_term
    }

    fn get_die(ext_dice: &ExtendedDamageDice, weapon_die: Option<DamageDice>) -> Result<DamageDice, CCError> {
        match ext_dice {
            ExtendedDamageDice::Basic(d) => Ok(*d),
            ExtendedDamageDice::WeaponDice => {
                if let Some(d) = weapon_die {
                    Ok(d)
                } else {
                    Err(CCError::NoWeaponSet)
                }
            },
            ExtendedDamageDice::SingleWeaponDie => {
                if let Some(d) = weapon_die {
                    Ok(ExtendedDamageDice::get_single_die(d))
                } else {
                    Err(CCError::NoWeaponSet)
                }
            },
        }
    }
}

impl From<DiceExprTerm> for DiceExpression {
    fn from(value: DiceExprTerm) -> Self {
        let mut de = DiceExpression::new();
        de.add_term(value);
        de
    }
}

impl From<(Vec<ExtendedDamageDice>, isize)> for DiceExpression {
    fn from(value: (Vec<ExtendedDamageDice>, isize)) -> Self {
        Self {
            dice_terms: value.0,
            const_term: value.1,
        }
    }
}

pub trait DiceExpr: Debug + From<DiceExprTerm> {
    fn add_term(&mut self, term: DiceExprTerm);
    fn get_dice_rv<P: RVProb>(&self, dmg_feats: &HashSet<DamageFeature>, weapon_dmg: Option<DamageDice>) -> Result<VecRandVar<P>, CCError>;
    fn get_const(&self) -> isize;

    fn get_base_dice_rv<P: RVProb>(&self) -> Result<VecRandVar<P>, CCError> {
        self.get_dice_rv(&HashSet::new(), None)
    }

    // healing is currently just negative damage
    fn get_heal_rv<P: RVProb> (&self) -> Result<VecRandVar<P>, CCError> {
        let rv_base = self.get_base_dice_rv();
        rv_base.map(|rv| rv.opposite_rv())
    }
}

impl DiceExpr for DiceExpression {
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

    fn get_dice_rv<P: RVProb>(&self, dmg_feats: &HashSet<DamageFeature>, weapon_dmg: Option<DamageDice>) -> Result<VecRandVar<P>, CCError> {
        let gwf = dmg_feats.contains(&DamageFeature::GWF);
        let mut rv: VecRandVar<P> = VecRandVar::new_constant(0).unwrap();
        for ext_dice in self.dice_terms.iter() {
            let dice = DiceExpression::get_die(ext_dice, weapon_dmg)?;
            if gwf {
                rv = rv.add_rv(&dice.get_rv_gwf());
            } else {
                rv = rv.add_rv(&dice.get_rv());
            }
        }
        Ok(rv)
    }

    fn get_const(&self) -> isize {
        self.const_term
    }
}
