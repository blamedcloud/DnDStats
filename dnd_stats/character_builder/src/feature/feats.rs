use std::clone::Clone;

use combat_core::ability_scores::Ability;
use combat_core::actions::{ActionName, ActionType, AttackType, CABuilder, CombatOption};
use combat_core::damage::{DamageTerm, ExpressionTerm, ExtendedDamageType};
use combat_core::resources::{RefreshBy, RefreshTiming, Resource, ResourceCap, ResourceName};

use crate::{CBError, Character, CharacterCO};
use crate::attributed_bonus::{BonusTerm, BonusType};
use crate::equipment::{Weapon, WeaponProperty};
use crate::feature::Feature;

pub struct GreatWeaponMaster;
impl GreatWeaponMaster {
    pub fn get_new_co(co: &CharacterCO) -> Option<CharacterCO> {
        if let CABuilder::WeaponAttack(wa) = &co.action {
            if wa.get_weapon().get_type().is_melee() && wa.get_weapon().has_property(WeaponProperty::Heavy) {
                let mut new_wa = wa.clone();
                new_wa.add_accuracy_bonus(BonusTerm::new_attr(BonusType::Constant(-5), String::from("gwm")));
                new_wa.get_damage_mut().add_base_dmg(DamageTerm::new(ExpressionTerm::Const(10), ExtendedDamageType::WeaponDamage));
                return Some(CombatOption::new(co.action_type, CABuilder::WeaponAttack(new_wa)));
            }
        }
        None
    }
}
impl Feature for GreatWeaponMaster {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let mut new_actions: Vec<(ActionName, CharacterCO)> = Vec::new();
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
        character.combat_actions.insert(ActionName::BonusGWMAttack, CombatOption::new(ActionType::BonusAction, CABuilder::AdditionalAttacks(1)));

        let mut res = Resource::new(ResourceCap::Soft(1), 0);
        res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToEmpty);
        res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToEmpty);
        character.resource_manager.add_perm(ResourceName::AN(ActionName::BonusGWMAttack), res);

        Ok(())
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

    pub fn get_new_co(co: &CharacterCO) -> Option<CharacterCO> {
        if let CABuilder::WeaponAttack(wa) = &co.action {
            if PolearmMaster::is_valid_weapon(wa.get_weapon()) {
                let new_wa = wa.as_pam_attack();
                return Some(CombatOption::new(ActionType::BonusAction, CABuilder::WeaponAttack(new_wa)));
            }
        }
        None
    }
}
impl Feature for PolearmMaster {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let mut new_actions: Vec<(ActionName, CharacterCO)> = Vec::new();
        for (ca, co) in character.combat_actions.iter() {
            match ca {
                ActionName::PrimaryAttack(at) => {
                    let new_co = PolearmMaster::get_new_co(co);
                    if new_co.is_some() {
                        new_actions.push((ActionName::BonusPAMAttack(*at), new_co.unwrap()));
                    }
                },
                _ => {}
            }
        }
        for (ca, co) in new_actions.into_iter() {
            character.combat_actions.insert(ca, co);
        }
        Ok(())
    }
}

pub struct Resilient(pub Ability);
impl Feature for Resilient {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let ability = character.ability_scores.get_score_mut(&self.0);
        ability.increase();
        ability.set_prof_save(true);
        Ok(())
    }
}

pub struct SharpShooter;
impl SharpShooter {
    pub fn get_new_co(co: &CharacterCO) -> Option<CharacterCO> {
        if let CABuilder::WeaponAttack(wa) = &co.action {
            if wa.get_weapon().get_type().is_ranged() {
                let mut new_wa = wa.clone();
                new_wa.add_accuracy_bonus(BonusTerm::new_attr(BonusType::Constant(-5), String::from("ss")));
                new_wa.get_damage_mut().add_base_dmg(DamageTerm::new(ExpressionTerm::Const(10), ExtendedDamageType::WeaponDamage));
                return Some(CombatOption::new(co.action_type, CABuilder::WeaponAttack(new_wa)));
            }
        }
        None
    }
}
impl Feature for SharpShooter {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let mut new_actions: Vec<(ActionName, CharacterCO)> = Vec::new();
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use num::{BigInt, BigRational, FromPrimitive};

    use combat_core::ability_scores::Ability;
    use combat_core::actions::{ActionName, AttackType, CABuilder};
    use combat_core::attack::{AccMRV64, AttackHitType};
    use rand_var::rv_traits::{NumRandVar, RandVar};
    use rand_var::rv_traits::sequential::Pair;
    use rand_var::RVBig;

    use crate::{Character, CharacterCO};
    use crate::classes::{ChooseSubClass, ClassName};
    use crate::classes::fighter::ChampionFighter;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::feats::{GreatWeaponMaster, PolearmMaster, Resilient, SharpShooter};
    use crate::tests::{get_dex_based, get_str_based};
    use crate::weapon_attack::WeaponAttack;

    fn get_attack(option: &CharacterCO) -> &WeaponAttack {
        if let CABuilder::WeaponAttack(wa) = &option.action {
            wa
        } else {
            panic!("should be an attack");
        }
    }

    #[test]
    fn resilient_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("gwf"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(Resilient(Ability::WIS)))).unwrap();
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
        fighter.level_up(ClassName::Fighter, vec!(Box::new(GreatWeaponMaster))).unwrap();
        let gwm_option = fighter.get_combat_option(ActionName::PrimaryAttack(AttackType::GWMAttack)).unwrap();
        let gwm_attack = get_attack(gwm_option);
        let acc: AccMRV64 = gwm_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 1), acc.lower_bound());
        assert_eq!(Pair(20, 20), acc.upper_bound());
        let dmg: RVBig = gwm_attack.get_damage().get_base_dmg(&HashSet::new(), vec!()).unwrap();
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
        fighter.level_up(ClassName::Fighter, vec!(Box::new(SharpShooter))).unwrap();
        let ss_option = fighter.get_combat_option(ActionName::PrimaryAttack(AttackType::SSAttack)).unwrap();
        let ss_attack = get_attack(ss_option);
        let acc: AccMRV64 = ss_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 1), acc.lower_bound());
        assert_eq!(Pair(20, 20), acc.upper_bound());
        let dmg: RVBig = ss_attack.get_damage().get_base_dmg(&HashSet::new(), vec!()).unwrap();
        assert_eq!(14, dmg.lower_bound());
        assert_eq!(21, dmg.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(35).unwrap(), BigInt::from_isize(2).unwrap()), dmg.expected_value());
    }

    #[test]
    fn pam_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::halberd(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("pam"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(PolearmMaster))).unwrap();
        let pam_option = fighter.get_combat_option(ActionName::BonusPAMAttack(AttackType::Normal)).unwrap();
        let pam_attack = get_attack(pam_option);
        let acc: AccMRV64 = pam_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 6), acc.lower_bound());
        assert_eq!(Pair(20, 25), acc.upper_bound());
        let dmg: RVBig = pam_attack.get_damage().get_base_dmg(&HashSet::new(), vec!()).unwrap();
        assert_eq!(4, dmg.lower_bound());
        assert_eq!(7, dmg.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(11).unwrap(), BigInt::from_isize(2).unwrap()), dmg.expected_value());
    }

    #[test]
    fn gwm_pam_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::halberd(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("gwm-pam"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(GreatWeaponMaster))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!()).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ChooseSubClass(ChampionFighter)))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new( PolearmMaster))).unwrap();
        assert!(fighter.has_combat_option(ActionName::PrimaryAttack(AttackType::Normal)));
        assert!(fighter.has_combat_option(ActionName::PrimaryAttack(AttackType::GWMAttack)));
        assert!(fighter.has_combat_option(ActionName::BonusPAMAttack(AttackType::Normal)));
        assert!(fighter.has_combat_option(ActionName::BonusPAMAttack(AttackType::GWMAttack)));
        let gwm_pam_option = fighter.get_combat_option(ActionName::BonusPAMAttack(AttackType::GWMAttack)).unwrap();
        let gwm_pam_attack = get_attack(gwm_pam_option);
        let acc: AccMRV64 = gwm_pam_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 1), acc.lower_bound());
        assert_eq!(Pair(20, 20), acc.upper_bound());
        let dmg: RVBig = gwm_pam_attack.get_damage().get_base_dmg(&HashSet::new(), vec!()).unwrap();
        assert_eq!(14, dmg.lower_bound());
        assert_eq!(17, dmg.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(31).unwrap(), BigInt::from_isize(2).unwrap()), dmg.expected_value());
    }

    #[test]
    fn pam_gwm_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::halberd(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("pam-gwm"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(PolearmMaster))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!()).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ChooseSubClass(ChampionFighter)))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(GreatWeaponMaster))).unwrap();
        assert!(fighter.has_combat_option(ActionName::PrimaryAttack(AttackType::Normal)));
        assert!(fighter.has_combat_option(ActionName::PrimaryAttack(AttackType::GWMAttack)));
        assert!(fighter.has_combat_option(ActionName::BonusPAMAttack(AttackType::Normal)));
        assert!(fighter.has_combat_option(ActionName::BonusPAMAttack(AttackType::GWMAttack)));
        let pam_gwm_option = fighter.get_combat_option(ActionName::BonusPAMAttack(AttackType::GWMAttack)).unwrap();
        let pam_gwm_attack = get_attack(pam_gwm_option);
        let acc: AccMRV64 = pam_gwm_attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 1), acc.lower_bound());
        assert_eq!(Pair(20, 20), acc.upper_bound());
        let dmg: RVBig = pam_gwm_attack.get_damage().get_base_dmg(&HashSet::new(), vec!()).unwrap();
        assert_eq!(14, dmg.lower_bound());
        assert_eq!(17, dmg.upper_bound());
        assert_eq!(BigRational::new(BigInt::from_isize(31).unwrap(), BigInt::from_isize(2).unwrap()), dmg.expected_value());
    }
}
