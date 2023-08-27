use std::collections::{BTreeMap, HashSet};
use combat_core::attack::{AccMRV, Attack, AttackHitType, AttackResult, D20Type};
use combat_core::damage::{DamageDice, DamageTerm, DamageType, ExpressionTerm, ExtendedDamageDice, ExtendedDamageType};
use rand_var::RandomVariable;
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::RVError;
use rand_var::rv_traits::sequential::Pair;
use crate::CBError;
use crate::damage_manager::DamageManager;

#[derive(Debug, Clone)]
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

impl<T: RVProb, E: From<RVError> + From<CBError>> Attack<T, E> for BasicAttack {
    fn get_dmg_map(&self, resistances: &HashSet<DamageType>) -> Result<BTreeMap<AttackResult, RandomVariable<T>>, E> {
        Ok(self.damage.get_attack_dmg_map(resistances)?)
    }

    fn get_acc_rv(&self, hit_type: AttackHitType) -> Result<AccMRV<T>, E> {
        let rv = hit_type.get_rv(&D20Type::D20);
        Ok(rv.into_mrv().map_keys(|roll| Pair(roll, roll + self.hit_bonus)))
    }
}
