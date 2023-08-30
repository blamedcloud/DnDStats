use std::collections::HashSet;

use rand_var::vec_rand_var::VecRandVar;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::rand_var::sequential::Pair;

use crate::{CCError, D20RollType, D20Type};
use crate::attack::{AccMRV, AtkDmgMap, Attack};
use crate::conditions::AttackDistance;
use crate::damage::{DamageDice, DamageManager, DamageTerm, DamageType, ExtendedDamageDice, ExtendedDamageType};
use crate::damage::dice_expr::{DiceExpression, DiceExprTerm};

#[derive(Debug, Clone)]
pub struct BasicAttack {
    damage: DamageManager<DiceExpression>,
    hit_bonus: isize,
    crit_lb: isize,
}

impl BasicAttack {
    pub fn new(hit_bonus: isize, dmg_type: DamageType, dmg_const: isize, dmg_die: DamageDice, num_dice: u8) -> Self {
        let mut damage = DamageManager::new();
        let e_dmg_type = ExtendedDamageType::Basic(dmg_type);
        damage.add_base_dmg(DamageTerm::new(DiceExprTerm::Const(dmg_const), e_dmg_type));
        let e_dmg_die = ExtendedDamageDice::Basic(dmg_die);
        damage.add_base_dmg(DamageTerm::new(DiceExprTerm::Dice(num_dice, e_dmg_die), e_dmg_type));
        Self {
            damage,
            hit_bonus,
            crit_lb: 20,
        }
    }

    pub fn prebuilt(damage: DamageManager<DiceExpression>, hit_bonus: isize, crit_lb: isize) -> Self {
        Self {
            damage,
            hit_bonus,
            crit_lb,
        }
    }

    pub fn set_crit_lb(&mut self, lb: isize) {
        if 1 < lb && lb <= 20 {
            self.crit_lb = lb;
        }
    }

    pub fn get_damage(&self) -> &DamageManager<DiceExpression> {
        &self.damage
    }
}

impl Attack for BasicAttack {
    fn get_miss_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<VecRandVar<T>, CCError> {
        Ok(self.damage.get_miss_dmg(resistances, bonus_dmg)?)
    }

    fn get_hit_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<VecRandVar<T>, CCError> {
        Ok(self.damage.get_base_dmg(resistances, bonus_dmg)?)
    }

    fn get_crit_dmg<T: RVProb>(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<VecRandVar<T>, CCError> {
        Ok(self.damage.get_crit_dmg(resistances, bonus_dmg)?)
    }

    fn get_acc_rv<T: RVProb>(&self, hit_type: D20RollType) -> Result<AccMRV<T>, CCError> {
        let rv = hit_type.get_rv(&D20Type::D20);
        Ok(rv.into_mrv().map_keys(|roll| Pair(roll, roll + self.hit_bonus)))
    }

    fn get_atk_range(&self) -> AttackDistance {
        // TODO: check this?
        AttackDistance::Within5Ft
    }

    fn get_crit_lb(&self) -> isize {
        self.crit_lb
    }

    fn get_hit_bonus(&self) -> isize {
        self.hit_bonus
    }

    fn get_dmg_map<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<AtkDmgMap<T>, CCError> {
        Ok(self.damage.get_attack_dmg_map(resistances)?)
    }
}
