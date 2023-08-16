use std::clone::Clone;
use crate::ability_scores::Ability;
use crate::attributed_bonus::{BonusTerm, BonusType};
use crate::Character;
use crate::combat::{ActionName, ActionType, AttackType, CombatAction, CombatOption};
use crate::damage::{DamageInstance, DamageTerm};
use crate::equipment::{Weapon, WeaponProperty};
use crate::feature::Feature;

pub struct GreatWeaponMaster;

impl GreatWeaponMaster {
    pub fn get_new_co(co: &CombatOption) -> Option<CombatOption> {
        if let CombatAction::Attack(wa) = &co.action {
            if wa.get_weapon().get_type().is_melee() && wa.get_weapon().has_property(WeaponProperty::Heavy) {
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
                        new_actions.push((ActionName::PrimaryAttack(AttackType::GWMAttack), new_co.unwrap()));
                    }
                },
                ActionName::OffhandAttack(AttackType::Normal) => {
                    let new_co = GreatWeaponMaster::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::OffhandAttack(AttackType::GWMAttack), new_co.unwrap()));
                    }
                },
                ActionName::BonusPAMAttack(AttackType::Normal) => {
                    let new_co = GreatWeaponMaster::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::BonusPAMAttack(AttackType::GWMAttack), new_co.unwrap()));
                    }
                }
                _ => {}
            }
        }
        for (ca, co) in new_actions.into_iter() {
            character.combat_actions.insert(ca, co);
        }
        character.combat_actions.insert(ActionName::BonusGWMAttack, CombatOption::new(ActionType::BonusAction, CombatAction::AdditionalAttacks(1)));
    }
}

pub struct PolearmMaster;

impl PolearmMaster {
    pub fn is_valid_weapon(weapon: &Weapon) -> bool {
        let name = weapon.get_name();
        if name == Weapon::QUARTERSTAFF || name == Weapon::HALBERD || name == Weapon::GLAIVE {
            true
        } else {
            false
        }
    }

    pub fn get_new_co(co: &CombatOption) -> Option<CombatOption> {
        if let CombatAction::Attack(wa) = &co.action {
            if PolearmMaster::is_valid_weapon(wa.get_weapon()) {
                let new_wa = wa.as_pam_attack();
                return Some(CombatOption::new(ActionType::BonusAction, CombatAction::Attack(new_wa)));
            }
        }
        None
    }
}

impl Feature for PolearmMaster {
    fn apply(&self, character: &mut Character) {
        let mut new_actions: Vec<(ActionName, CombatOption)> = Vec::new();
        for (ca, co) in character.combat_actions.iter() {
            match ca {
                ActionName::PrimaryAttack(at) => {
                    let new_co = PolearmMaster::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::BonusPAMAttack(*at),new_co.unwrap()));
                    }
                },
                _ => {}
            }
        }
        for (ca, co) in new_actions.into_iter() {
            character.combat_actions.insert(ca, co);
        }
    }
}

pub struct Resilient(Ability);

impl Feature for Resilient {
    fn apply(&self, character: &mut Character) {
        let ability = character.ability_scores.get_score_mut(&self.0);
        ability.increase();
        ability.set_prof_save(true);
    }
}

pub struct SharpShooter;

impl SharpShooter {
    pub fn get_new_co(co: &CombatOption) -> Option<CombatOption> {
        if let CombatAction::Attack(wa) = &co.action {
            if wa.get_weapon().get_type().is_ranged() {
                let mut new_wa = wa.clone();
                new_wa.add_accuracy_bonus(BonusTerm::new_attr(BonusType::Constant(-5), String::from("ss")));
                new_wa.get_damage_mut().add_base_dmg(DamageTerm::new(DamageInstance::Const(10), *wa.get_weapon().get_dmg_type()));
                return Some(CombatOption::new(co.action_type, CombatAction::Attack(new_wa)));
            }
        }
        None
    }
}

impl Feature for SharpShooter {
    fn apply(&self, character: &mut Character) {
        let mut new_actions: Vec<(ActionName, CombatOption)> = Vec::new();
        for (ca, co) in character.combat_actions.iter() {
            match ca {
                ActionName::PrimaryAttack(AttackType::Normal) => {
                    let new_co = SharpShooter::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::PrimaryAttack(AttackType::SSAttack), new_co.unwrap()));
                    }
                },
                ActionName::OffhandAttack(AttackType::Normal) => {
                    let new_co = SharpShooter::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::OffhandAttack(AttackType::SSAttack), new_co.unwrap()));
                    }
                },
                _ => {}
            }
        }
        for (ca, co) in new_actions.into_iter() {
            character.combat_actions.insert(ca, co);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use num::{BigInt, BigRational, FromPrimitive};
    use rand_var::RandomVariable;
    use rand_var::rv_traits::{NumRandVar, RandVar};
    use rand_var::rv_traits::sequential::Pair;
    use crate::{Character, HitDice};
    use crate::ability_scores::Ability;
    use crate::combat::{ActionName, AttackType, CombatAction};
    use crate::combat::attack::AttackHitType;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::feats::{GreatWeaponMaster, Resilient, SharpShooter};
    use crate::tests::{get_dex_based, get_str_based};

    #[test]
    fn resilient_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("gwf"), get_str_based(), equipment);
        fighter.level_up(HitDice::D10, vec!(Box::new(Resilient(Ability::WIS))));
        let wis = fighter.ability_scores.wisdom;
        assert_eq!(14, wis.get_score());
        assert!(wis.is_prof_save());
    }

    #[test]
    fn gwm_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("gwf"), get_str_based(), equipment);
        fighter.level_up(HitDice::D10, vec!(Box::new(GreatWeaponMaster)));
        let gwm_option = fighter.get_combat_option(ActionName::PrimaryAttack(AttackType::GWMAttack)).unwrap();
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

    #[test]
    fn sharpshooter_test() {
        let equipment = Equipment::new(
            Armor::leather(),
            Weapon::longbow(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("sharpshooter"), get_dex_based(), equipment);
        fighter.level_up(HitDice::D10, vec!(Box::new(SharpShooter)));
        let ss_option = fighter.get_combat_option(ActionName::PrimaryAttack(AttackType::SSAttack)).unwrap();
        let ss_attack;
        if let CombatAction::Attack(wa) = &ss_option.action {
            ss_attack = wa;
        } else {
            panic!("should be an attack");
        }
        let acc = ss_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 1), acc.lower_bound());
        assert_eq!(Pair(20, 20), acc.upper_bound());
        let dmg: RandomVariable<BigRational> = ss_attack.get_damage().get_base_dmg(&HashSet::new()).unwrap();
        assert_eq!(14, dmg.lower_bound());
        assert_eq!(21, dmg.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(35).unwrap(), BigInt::from_isize(2).unwrap()), dmg.expected_value());
    }

    #[test]
    fn pam_test() {
        todo!()
    }

    #[test]
    fn gwm_pam_test() {
        todo!()
    }

    #[test]
    fn pam_gwm_test() {
        todo!()
    }
}
