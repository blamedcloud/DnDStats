use std::collections::{BTreeMap, HashSet};
use rand_var::RandomVariable;
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::sequential::Pair;
use crate::CBError;
use crate::combat::attack::{AccMRV, Attack, AttackHitType, AttackResult, D20Type};
use crate::damage::{DamageDice, DamageManager, DamageTerm, DamageType, ExpressionTerm, ExtendedDamageDice, ExtendedDamageType};

#[derive(Clone)]
pub struct BasicAttack {
    damage: DamageManager,
    hit_bonus: isize,
}

impl BasicAttack {
    pub fn new(hit_bonus: isize, dmg_type: DamageType, dmg_const: isize, dmg_die: DamageDice, num_dice: u8) -> Self {
        let mut damage = DamageManager::new();
        let e_dmg_type = ExtendedDamageType::Basic(dmg_type);
        damage.add_base_dmg(DamageTerm::new(ExpressionTerm::Const(dmg_const), e_dmg_type));
        let e_dmg_die = ExtendedDamageDice::Basic(dmg_die);
        damage.add_base_dmg(DamageTerm::new(ExpressionTerm::Dice(num_dice, e_dmg_die), e_dmg_type));
        Self {
            damage,
            hit_bonus
        }
    }
}

impl Attack for BasicAttack {
    fn get_dmg_map<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<BTreeMap<AttackResult, RandomVariable<T>>, CBError> {
        self.damage.get_attack_dmg_map(resistances)
    }

    fn get_accuracy_rv<T: RVProb>(&self, hit_type: AttackHitType) -> Result<AccMRV<T>, CBError> {
        let rv = hit_type.get_rv(&D20Type::D20);
        Ok(rv.into_mrv().map_keys(|roll| Pair(roll, roll + self.hit_bonus)))
    }
}
