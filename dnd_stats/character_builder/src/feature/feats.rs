use crate::attributed_bonus::{BonusTerm, BonusType};
use crate::Character;
use crate::combat::{ActionName, ActionType, AttackType, CombatAction, CombatOption};
use crate::damage::{DamageInstance, DamageTerm};
use crate::equipment::WeaponProperty;
use crate::feature::Feature;

pub struct GreatWeaponMaster;

impl GreatWeaponMaster {
    pub fn get_new_co(co: &CombatOption) -> Option<CombatOption> {
        if let CombatAction::Attack(wa) = &co.action {
            if wa.get_weapon().has_property(WeaponProperty::Heavy) {
                let mut new_wa = wa.clone();
                new_wa.add_accuracy_bonus(BonusTerm::new_attr(BonusType::Constant(-5), String::from("gwm")));
                new_wa.get_damage_mut().add_base_dmg(DamageTerm::new(DamageInstance::Const(10), *wa.get_weapon().get_dmg_type()));
                return Some(CombatOption::new(co.action_type, CombatAction::Attack(new_wa)));
            }
        }
        None
    }
}

impl Feature for GreatWeaponMaster {
    fn apply(&self, character: &mut Character) {
        let mut new_actions: Vec<(ActionName, CombatOption)> = Vec::new();
        for (ca, co) in character.combat_actions.iter() {
            match ca {
                ActionName::PrimaryAttack(AttackType::Normal) => {
                    let new_co = GreatWeaponMaster::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::PrimaryAttack(AttackType::GreatWeaponMaster), new_co.unwrap()));
                    }
                },
                ActionName::OffhandAttack(AttackType::Normal) => {
                    let new_co = GreatWeaponMaster::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::OffhandAttack(AttackType::GreatWeaponMaster), new_co.unwrap()));
                    }
                },
                _ => {}
            }
        }
        for (ca, co) in new_actions.into_iter() {
            character.combat_actions.insert(ca, co);
        }
        character.combat_actions.insert(ActionName::BonusGWMAttack, CombatOption::new(ActionType::BonusAction, CombatAction::AdditionalAttacks(1)));
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use num::{BigRational, FromPrimitive};
    use rand_var::RandomVariable;
    use rand_var::rv_traits::{NumRandVar, RandVar};
    use rand_var::rv_traits::sequential::Pair;
    use crate::{Character, HitDice};
    use crate::combat::{ActionName, AttackType, CombatAction};
    use crate::combat::attack::AttackHitType;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::feats::GreatWeaponMaster;
    use crate::tests::get_str_based;

    #[test]
    fn gwm_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("gwf"), get_str_based(), equipment);
        fighter.level_up(HitDice::D10, vec!(Box::new(GreatWeaponMaster)));
        let gwm_option = fighter.get_combat_option(ActionName::PrimaryAttack(AttackType::GreatWeaponMaster)).unwrap();
        let gwm_attack;
        if let CombatAction::Attack(wa) = &gwm_option.action {
            gwm_attack = wa;
        } else {
            panic!("should be an attack");
        }
        let acc = gwm_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 1), acc.lower_bound());
        assert_eq!(Pair(20, 20), acc.upper_bound());
        let dmg: RandomVariable<BigRational> = gwm_attack.get_damage().get_base_dmg(&HashSet::new()).unwrap();
        assert_eq!(15, dmg.lower_bound());
        assert_eq!(25, dmg.upper_bound());
        assert_eq!(BigRational::from_isize(20).unwrap(), dmg.expected_value());
    }

}
