use std::ops::Add;
use crate::ability_scores::AbilityScores;
use crate::equipment::Equipment;

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
}

impl Character {
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
}

#[cfg(test)]
mod tests {
    use crate::equipment::{Armor, OffHand, Weapon};
    use super::*;

    pub fn get_test_fighter() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = AbilityScores::new(16,12,16,8,13,10);
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        Character {
            name,
            ability_scores,
            level: 1,
            equipment,
        }
    }

    #[test]
    fn basic_character_test() {
        let fighter = get_test_fighter();
        assert_eq!("FighterMan", fighter.get_name());
        assert_eq!(3, fighter.get_ability_scores().strength.get_mod());
        assert_eq!(2, fighter.get_prof_bonus());
        assert_eq!("Greatsword", fighter.get_equipment().get_primary_weapon().get_name());
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
