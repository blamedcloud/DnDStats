use std::rc::Rc;
use crate::attributed_bonus::{BonusTerm, BonusType, CharacterDependant};
use crate::{CBError, Character};
use crate::combat::attack::{HandType, NumHands};
use crate::combat::CombatAction;
use crate::damage::{DamageFeature, ExtendedDamageType};
use crate::equipment::ArmorType;
use crate::feature::Feature;

pub enum FightingStyles {
    Archery,
    Defense,
    Dueling,
    GreatWeaponFighting,
    //Protection,
    TwoWeaponFighting,
}

pub struct FightingStyle(pub FightingStyles);

impl FightingStyle {
    pub fn archery(character: &mut Character) {
        for (_, co) in character.combat_actions.iter_mut() {
            if let CombatAction::Attack(attack) = &mut co.action {
                if attack.get_weapon().get_type().is_ranged() {
                    attack.add_accuracy_bonus(BonusTerm::new_attr(BonusType::Constant(2), String::from("archery FS")));
                }
            }
        }
    }

    pub fn defense(character: &mut Character) {
        if character.equipment.get_armor().get_armor_type() != &ArmorType::NoArmor {
            character.armor_class.add_term(BonusTerm::new_attr(BonusType::Constant(1), String::from("defense FS")));
        }
    }

    pub fn dueling(character: &mut Character) {
        for (_, co) in character.combat_actions.iter_mut() {
            if let CombatAction::Attack(attack) = &mut co.action {
                if attack.get_num_hands() == &NumHands::OneHand && attack.get_weapon().get_type().is_melee() {
                    let dmg: CharacterDependant = Rc::new(|chr| {
                        if chr.get_equipment().get_secondary_weapon().is_none() {
                            2
                        } else {
                            0
                        }
                    });
                    attack.get_damage_mut().add_base_char_dmg(
                        ExtendedDamageType::WeaponDamage,
                        BonusTerm::new_attr(BonusType::Dependant(dmg), String::from("dueling FS"))
                    );
                }
            }
        }
    }

    pub fn gwf(character: &mut Character) {
        for (_, co) in character.combat_actions.iter_mut() {
            if let CombatAction::Attack(attack) = &mut co.action {
                if attack.get_num_hands() == &NumHands::TwoHand && attack.get_weapon().get_type().is_melee() {
                    attack.get_damage_mut().add_damage_feature(DamageFeature::GWF);
                }
            }
        }
    }

    pub fn twf(character: &mut Character) {
        for (_, co) in character.combat_actions.iter_mut() {
            if let CombatAction::Attack(attack) = &mut co.action {
                if attack.get_hand_type() == &HandType::OffHand {
                    let ability = *attack.get_ability();
                    attack.get_damage_mut().add_base_char_dmg(
                        ExtendedDamageType::WeaponDamage,
                        BonusTerm::new_attr(BonusType::Modifier(ability), String::from("twf FS"))
                    );
                }
            }
        }
    }
}

impl Feature for FightingStyle {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        match self.0 {
            FightingStyles::Archery => FightingStyle::archery(character),
            FightingStyles::Defense => FightingStyle::defense(character),
            FightingStyles::Dueling => FightingStyle::dueling(character),
            FightingStyles::GreatWeaponFighting => FightingStyle::gwf(character),
            FightingStyles::TwoWeaponFighting => FightingStyle::twf(character),
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use num::{BigInt, BigRational};
    use rand_var::{BigRV, RandomVariable};
    use rand_var::rv_traits::{NumRandVar, RandVar};
    use rand_var::rv_traits::sequential::Pair;
    use crate::Character;
    use crate::classes::ClassName;
    use crate::combat::attack::AttackHitType;
    use crate::equipment::{ACSource, Armor, Equipment, OffHand, Weapon};
    use crate::tests::{get_dex_based, get_str_based};
    use super::*;

    #[test]
    fn archery_test() {
        let equipment = Equipment::new(
            Armor::leather(),
            Weapon::longbow(),
            OffHand::Free,
        );
        let mut archer = Character::new(String::from("archer"), get_dex_based(), equipment);
        archer.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::Archery)))).unwrap();
        let attack = archer.get_basic_attack().unwrap();
        let acc = attack.get_accuracy_rv(AttackHitType::Normal).unwrap();
        assert_eq!(Pair(1, 8), acc.lower_bound());
        assert_eq!(Pair(20, 27), acc.upper_bound());
    }

    #[test]
    fn defense_test() {
        let equipment = Equipment::new(
            Armor::plate(),
            Weapon::longsword(),
            OffHand::Shield(ACSource::shield())
        );
        let mut fighter = Character::new(String::from("armored"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::Defense)))).unwrap();
        assert_eq!(21, fighter.get_ac());
    }

    #[test]
    fn dueling_test() {
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::rapier(),
            OffHand::Shield(ACSource::shield())
        );
        let mut fighter = Character::new(String::from("duelist"), get_dex_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::Dueling)))).unwrap();
        let dmg: BigRV = fighter.get_basic_attack().unwrap().get_damage().get_base_dmg(&HashSet::new()).unwrap();
        assert_eq!(6, dmg.lower_bound());
        assert_eq!(13, dmg.upper_bound());
        assert_eq!(BigRational::new(BigInt::from(19), BigInt::from(2)), dmg.expected_value());
    }

    #[test]
    fn gwf_test() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("gwf"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dmg: BigRV = fighter.get_basic_attack().unwrap().get_damage().get_base_dmg(&HashSet::new()).unwrap();
        assert_eq!(5, dmg.lower_bound());
        assert_eq!(15, dmg.upper_bound());
        let rv: BigRV = RandomVariable::new_dice_reroll(6, 2).unwrap().multiple(2).add_const(3);
        assert_eq!(dmg, rv);
    }

    #[test]
    fn twf_test() {
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::shortsword(),
            OffHand::Weapon(Weapon::shortsword())
        );
        let mut fighter = Character::new(String::from("kirito"), get_dex_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::TwoWeaponFighting)))).unwrap();
        let main_dmg: BigRV = fighter.get_basic_attack().unwrap().get_damage().get_base_dmg(&HashSet::new()).unwrap();
        let off_dmg: BigRV = fighter.get_offhand_attack().unwrap().get_damage().get_base_dmg(&HashSet::new()).unwrap();
        assert_eq!(main_dmg, off_dmg);
    }
}
