use std::cmp::min;
use std::ops::Add;
use crate::ability_scores::AbilityScores;
use crate::attributed_bonus::{AttributedBonus, BonusTerm, BonusType, CharacterDependant};
use crate::equipment::{ArmorType, Equipment};

pub mod ability_scores;
pub mod attributed_bonus;
pub mod damage;
pub mod equipment;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Feet(i32);
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Squares(i32);

impl Add<Squares> for Squares {
    type Output = Squares;

    fn add(self, other: Squares) -> Squares {
        Squares(self.0 + other.0)
    }
}

impl Add<Squares> for Feet {
    type Output = Feet;

    fn add(self, other: Squares) -> Feet {
        Feet(self.0 + (other.0 * 5))
    }
}

impl Add<Feet> for Feet {
    type Output = Feet;

    fn add(self, other: Feet) -> Feet {
        Feet(self.0 + other.0)
    }
}

pub struct Character {
    name: String,
    ability_scores: AbilityScores,
    level: u8,
    equipment: Equipment,
    armor_class: AttributedBonus,
}

impl Character {
    pub fn new(name: String, ability_scores: AbilityScores, equipment: Equipment) -> Self {
        let mut character = Character {
            name,
            ability_scores,
            level: 1,
            equipment,
            armor_class: AttributedBonus::new(String::from("AC")),
        };
        character.calc_ac();
        character
    }

    fn calc_ac(&mut self) {
        self.armor_class.reset();
        // base armor values
        let armor_ac: CharacterDependant = Box::new(
            |chr| chr.get_equipment().get_armor().get_ac_source().get_base_ac() as i32
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(armor_ac),
            String::from("base ac"),
            String::from("armor")
        ));
        let armor_mb: CharacterDependant = Box::new(
            |chr| chr.get_equipment().get_armor().get_ac_source().get_magic_bonus().unwrap_or(0) as i32
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(armor_mb),
            String::from("magic bonus"),
            String::from("armor")
        ));

        // base shield values
        let shield_ac: CharacterDependant = Box::new(
            |chr| chr.get_equipment().get_shield().map(|acs| acs.get_base_ac() as i32).unwrap_or(0)
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(shield_ac),
            String::from("shield ac"),
            String::from("shield")
        ));
        let shield_mb: CharacterDependant = Box::new(
            |chr| chr.get_equipment().get_shield().map(|acs| acs.get_magic_bonus().unwrap_or(0) as i32).unwrap_or(0)
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(shield_mb),
            String::from("magic bonus"),
            String::from("shield")
        ));

        // dex contribution
        let dex_contr: CharacterDependant = Box::new(|chr| {
           match &chr.get_equipment().get_armor().get_armor_type() {
               ArmorType::HeavyArmor => 0,
               ArmorType::MediumArmor => min(2, chr.get_ability_scores().dexterity.get_mod() as i32),
               _ => chr.get_ability_scores().dexterity.get_mod() as i32
           }
        });
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(dex_contr),
            String::from("dex contribution"),
            String::from("armor type")
        ));
        // TODO: other AC-modifying features (e.g. Defense fighting style)
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_ability_scores(&self) -> &AbilityScores {
        &self.ability_scores
    }
    pub fn get_ability_scores_mut(&mut self) -> &mut AbilityScores {
        &mut self.ability_scores
    }

    pub fn get_level(&self) -> u8 {
        self.level
    }
    pub fn get_prof_bonus(&self) -> u8 {
        (self.level + 3)/4 + 1
    }

    pub fn get_equipment(&self) -> &Equipment {
        &self.equipment
    }
    pub fn get_equipment_mut(&mut self) -> &mut Equipment {
        &mut self.equipment
    }

    pub fn get_ac(&self) -> i32 {
        self.armor_class.get_value(&self)
    }
}

#[cfg(test)]
mod tests {
    use crate::equipment::{ACSource, Armor, OffHand, Weapon};
    use super::*;

    pub fn get_test_fighter() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = AbilityScores::new(16,12,16,8,13,10);
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        Character::new(name, ability_scores, equipment)
    }

    #[test]
    fn basic_character_test() {
        let fighter = get_test_fighter();
        assert_eq!("FighterMan", fighter.get_name());
        assert_eq!(3, fighter.get_ability_scores().strength.get_mod());
        assert_eq!(2, fighter.get_prof_bonus());
        assert_eq!("Greatsword", fighter.get_equipment().get_primary_weapon().get_name());
        assert_eq!(16, fighter.get_ac());
    }

    #[test]
    fn ac_mut_test() {
        let mut dex_fighter = Character::new(
            String::from("DexMan"),
            AbilityScores::new(13, 16, 16, 12, 10, 8),
            Equipment::new(Armor::leather(), Weapon::rapier(), OffHand::Free)
        );
        assert_eq!(14, dex_fighter.get_ac());
        dex_fighter.get_equipment_mut().set_armor(Armor::chain_mail());
        assert_eq!(16, dex_fighter.get_ac());
        dex_fighter.get_equipment_mut().set_armor(Armor::half_plate());
        assert_eq!(17, dex_fighter.get_ac());
        dex_fighter.get_equipment_mut().set_off_hand(OffHand::Shield(ACSource::shield()));
        assert_eq!(19, dex_fighter.get_ac());
        if let OffHand::Shield(shield) = dex_fighter.get_equipment_mut().get_off_hand_mut() {
            shield.set_magic_bonus(1);
        }
        assert_eq!(20, dex_fighter.get_ac());
    }

    #[test]
    fn mut_character_test() {
        let mut fighter = get_test_fighter();
        assert_eq!(1, fighter.get_ability_scores().wisdom.get_mod());
        fighter.get_ability_scores_mut().wisdom.increase();
        assert_eq!(2, fighter.get_ability_scores().wisdom.get_mod());

        assert_eq!(None, fighter.get_equipment().get_primary_weapon().get_magic_bonus());
        fighter.get_equipment_mut().get_primary_weapon_mut().set_magic_bonus(1);
        assert_eq!(1, fighter.get_equipment().get_primary_weapon().get_magic_bonus().unwrap());
    }
}
