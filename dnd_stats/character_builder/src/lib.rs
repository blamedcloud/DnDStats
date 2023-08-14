use std::cmp::min;
use std::ops::Add;
use std::rc::Rc;
use rand_var::rv_traits::RVError;
use crate::ability_scores::AbilityScores;
use crate::attributed_bonus::{AttributedBonus, BonusTerm, BonusType, CharacterDependant};
use crate::combat::{ActionManager, ActionNames, CombatAction, create_action_manager};
use crate::combat::attack::WeaponAttack;
use crate::equipment::{ArmorType, Equipment};
use crate::feature::Feature;

pub mod ability_scores;
pub mod attributed_bonus;
pub mod combat;
pub mod damage;
pub mod equipment;
pub mod feature;

#[derive(Debug)]
pub enum CBError {
    NoCache,
    RVError(RVError),
    Other(String),
}

impl From<RVError> for CBError {
    fn from(value: RVError) -> Self {
        CBError::RVError(value)
    }
}

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

#[derive(Clone)]
pub struct Character {
    name: String,
    ability_scores: AbilityScores,
    level: u8,
    equipment: Equipment,
    armor_class: AttributedBonus,
    combat_actions: ActionManager,
}

impl Character {
    pub fn new(name: String, ability_scores: AbilityScores, equipment: Equipment) -> Self {
        let mut character = Character {
            name,
            ability_scores,
            level: 0,
            equipment,
            armor_class: AttributedBonus::new(String::from("AC")),
            combat_actions: ActionManager::new(),
        };
        character.calc_ac();
        character.combat_actions = create_action_manager(&character);
        character
    }

    fn calc_ac(&mut self) {
        self.armor_class.reset();
        // base armor values
        let armor_ac: CharacterDependant = Rc::new(
            |chr| chr.get_equipment().get_armor().get_ac_source().get_base_ac() as i32
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(armor_ac),
            String::from("base ac"),
            String::from("armor")
        ));
        let armor_mb: CharacterDependant = Rc::new(
            |chr| chr.get_equipment().get_armor().get_ac_source().get_magic_bonus().unwrap_or(0) as i32
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(armor_mb),
            String::from("magic bonus"),
            String::from("armor")
        ));

        // base shield values
        let shield_ac: CharacterDependant = Rc::new(
            |chr| chr.get_equipment().get_shield().map(|acs| acs.get_base_ac() as i32).unwrap_or(0)
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(shield_ac),
            String::from("shield ac"),
            String::from("shield")
        ));
        let shield_mb: CharacterDependant = Rc::new(
            |chr| chr.get_equipment().get_shield().map(|acs| acs.get_magic_bonus().unwrap_or(0) as i32).unwrap_or(0)
        );
        self.armor_class.add_term(BonusTerm::new_name_attr(
            BonusType::Dependant(shield_mb),
            String::from("magic bonus"),
            String::from("shield")
        ));

        // dex contribution
        let dex_contr: CharacterDependant = Rc::new(|chr| {
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
    }

    fn cache_self(&mut self) {
        // having to clone self is a bit gross, but is
        // necessary because you can't pass a mutable
        // &Character into attack::cache_char_vals
        let clone = self.clone();
        for (_, co) in self.combat_actions.iter_mut() {
            if let CombatAction::Attack(a) = &mut co.action {
                a.cache_char_vals(&clone);
            }
        }
    }

    pub fn level_up(&mut self, features: Vec<Box<dyn Feature>>) {
        for feat in features {
            feat.apply(self);
        }
        self.level += 1;
        self.cache_self();
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_ability_scores(&self) -> &AbilityScores {
        &self.ability_scores
    }
    // fn get_ability_scores_mut(&mut self) -> &mut AbilityScores {
    //     &mut self.ability_scores
    // }

    pub fn get_level(&self) -> u8 {
        self.level
    }
    pub fn get_prof_bonus(&self) -> u8 {
        (self.level + 3)/4 + 1
    }

    pub fn get_equipment(&self) -> &Equipment {
        &self.equipment
    }
    // fn get_equipment_mut(&mut self) -> &mut Equipment {
    //     &mut self.equipment
    // }

    pub fn get_ac(&self) -> i32 {
        self.armor_class.get_value(&self)
    }

    pub fn get_basic_attack(&self) -> Option<&WeaponAttack> {
        self.combat_actions.get(&ActionNames::BasicAttack).and_then(|co| {
            if let CombatAction::Attack(wa) = &co.action {
                Some(wa)
            } else {
                None
            }
        })
    }

    pub fn get_offhand_attack(&self) -> Option<&WeaponAttack> {
        self.combat_actions.get(&ActionNames::OffhandAttack).and_then(|co| {
            if let CombatAction::Attack(wa) = &co.action {
                Some(wa)
            } else {
                None
            }
        })
    }

    pub fn get_action_manager(&self) -> &ActionManager {
        &self.combat_actions
    }
}

#[cfg(test)]
mod tests {
    use crate::equipment::{ACSource, Armor, OffHand, Weapon};
    use super::*;

    pub fn get_str_based() -> AbilityScores {
        AbilityScores::new(16,12,16,8,13,10)
    }

    pub fn get_dex_based() -> AbilityScores {
        AbilityScores::new(12,16,16,8,13,10)
    }

    pub fn get_test_fighter() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        let mut fighter = Character::new(name, ability_scores, equipment);
        fighter.level_up(vec!());
        fighter
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
        dex_fighter.equipment.set_armor(Armor::chain_mail());
        assert_eq!(16, dex_fighter.get_ac());
        dex_fighter.equipment.set_armor(Armor::half_plate());
        assert_eq!(17, dex_fighter.get_ac());
        dex_fighter.equipment.set_off_hand(OffHand::Shield(ACSource::shield()));
        assert_eq!(19, dex_fighter.get_ac());
        if let OffHand::Shield(shield) = dex_fighter.equipment.get_off_hand_mut() {
            shield.set_magic_bonus(1);
        }
        assert_eq!(20, dex_fighter.get_ac());
    }

    #[test]
    fn mut_character_test() {
        let mut fighter = get_test_fighter();
        assert_eq!(1, fighter.get_ability_scores().wisdom.get_mod());
        fighter.ability_scores.wisdom.increase();
        assert_eq!(2, fighter.get_ability_scores().wisdom.get_mod());

        assert_eq!(None, fighter.get_equipment().get_primary_weapon().get_magic_bonus());
        fighter.equipment.get_primary_weapon_mut().set_magic_bonus(1);
        assert_eq!(1, fighter.get_equipment().get_primary_weapon().get_magic_bonus().unwrap());
    }
}
