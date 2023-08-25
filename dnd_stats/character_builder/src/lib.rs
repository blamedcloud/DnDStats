use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::rc::Rc;
use rand_var::rv_traits::RVError;
use crate::ability_scores::AbilityScores;
use crate::attributed_bonus::{AttributedBonus, BonusTerm, BonusType, CharacterDependant};
use crate::classes::{ClassName, SubClass};
use crate::combat::{ActionManager, ActionName, AttackType, CombatAction, CombatOption, create_character_am};
use crate::combat::attack::weapon_attack::WeaponAttack;
use crate::damage::DamageType;
use crate::equipment::{ArmorType, Equipment};
use crate::feature::Feature;
use crate::resources::{create_basic_rm, ResourceManager};

pub mod ability_scores;
pub mod attributed_bonus;
pub mod classes;
pub mod combat;
pub mod damage;
pub mod equipment;
pub mod feature;
pub mod resources;

#[derive(Debug)]
pub enum CBError {
    NoCache,
    NoWeaponSet,
    NotImplemented,
    NoClassSet,
    NoSubClassSet,
    InvalidLevel,
    RVError(RVError),
    Other(String),
}

impl From<RVError> for CBError {
    fn from(value: RVError) -> Self {
        CBError::RVError(value)
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct Feet(i32);
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct Square(i32);

impl Add<Square> for Square {
    type Output = Square;

    fn add(self, other: Square) -> Square {
        Square(self.0 + other.0)
    }
}

impl Add<Square> for Feet {
    type Output = Feet;

    fn add(self, other: Square) -> Feet {
        Feet(self.0 + (other.0 * 5))
    }
}

impl Add<Feet> for Feet {
    type Output = Feet;

    fn add(self, other: Feet) -> Feet {
        Feet(self.0 + other.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HitDice {
    D6,
    D8,
    D10,
    D12,
}

impl HitDice {
    pub fn get_max(&self) -> isize {
        match self {
            HitDice::D6 => 6,
            HitDice::D8 => 8,
            HitDice::D10 => 10,
            HitDice::D12 => 12,
        }
    }

    pub fn get_per_lvl(&self) -> isize {
        match self {
            HitDice::D6 => 4,
            HitDice::D8 => 5,
            HitDice::D10 => 6,
            HitDice::D12 => 7,
        }
    }
}

#[derive(Clone)]
pub struct Character {
    name: String,
    ability_scores: AbilityScores,
    level: u8,
    class_lvls: Vec<ClassName>,
    sub_classes: HashMap<ClassName, Rc<dyn SubClass>>,
    equipment: Equipment,
    armor_class: AttributedBonus,
    combat_actions: ActionManager,
    resource_manager: ResourceManager,
    resistances: HashSet<DamageType>,
}

impl Character {
    pub fn new(name: String, ability_scores: AbilityScores, equipment: Equipment) -> Self {
        let mut character = Character {
            name,
            ability_scores,
            level: 0,
            class_lvls: Vec::new(),
            sub_classes: HashMap::new(),
            equipment,
            armor_class: AttributedBonus::new(String::from("AC")),
            combat_actions: ActionManager::new(),
            resource_manager: create_basic_rm(),
            resistances: HashSet::new(),
        };
        character.calc_ac();
        character.combat_actions = create_character_am(&character);
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
            match &mut co.action {
                CombatAction::WeaponAttack(wa) => wa.cache_char_vals(&clone),
                CombatAction::SelfHeal(de) => de.cache_char_terms(&clone),
                _ => {}
            }
        }
    }

    pub fn level_up(&mut self, class: ClassName, features: Vec<Box<dyn Feature>>) -> Result<(), CBError> {
        self.level += 1;
        if self.level == 1 {
            class.get_save_profs().apply(self)?;
        }
        for feat in features {
            feat.apply(self)?;
        }
        self.class_lvls.push(class);
        let class_level = self.get_class_level(class);
        let class_features = class.get_class()?.get_static_features(class_level)?;
        for feat in class_features {
            feat.apply(self)?;
        }
        self.cache_self();
        Ok(())
    }

    pub fn level_up_basic(&mut self) -> Result<(), CBError> {
        let class = self.class_lvls.last().ok_or(CBError::NoClassSet)?;
        self.level_up(*class, Vec::new())?;
        Ok(())
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

    pub fn get_class_levels(&self) -> &Vec<ClassName> {
        &self.class_lvls
    }
    pub fn get_class_level(&self, class: ClassName) -> u8 {
        self.class_lvls.iter().filter(|c| **c == class).map(|_| 1).sum()
    }

    pub fn get_sub_class(&self, class: ClassName) -> Result<Rc<dyn SubClass>, CBError> {
        if let Some(sub) = self.sub_classes.get(&class) {
            Ok(sub.clone())
        } else {
            Err(CBError::NoSubClassSet)
        }
    }

    pub fn get_level(&self) -> u8 {
        self.level
    }
    pub fn get_prof_bonus(&self) -> u8 {
        (self.level + 3)/4 + 1
    }

    pub fn get_max_hp(&self) -> isize {
        let mut max_hp = 0;
        let mut hd_iter = self.class_lvls.iter().map(|c| c.get_hit_die());
        let first_hd = hd_iter.next();
        if first_hd.is_some() {
            max_hp += first_hd.unwrap().get_max();
        }
        for hd in hd_iter {
            max_hp += hd.get_per_lvl();
        }
        max_hp += (self.level as isize) * (self.ability_scores.constitution.get_mod() as isize);
        max_hp
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

    pub fn get_weapon_attack(&self) -> Option<&WeaponAttack> {
        self.combat_actions.get(&ActionName::PrimaryAttack(AttackType::Normal)).and_then(|co| {
            if let CombatAction::WeaponAttack(wa) = &co.action {
                Some(wa)
            } else {
                None
            }
        })
    }

    pub fn get_offhand_attack(&self) -> Option<&WeaponAttack> {
        self.combat_actions.get(&ActionName::OffhandAttack(AttackType::Normal)).and_then(|co| {
            if let CombatAction::WeaponAttack(wa) = &co.action {
                Some(wa)
            } else {
                None
            }
        })
    }

    pub fn get_action_manager(&self) -> &ActionManager {
        &self.combat_actions
    }

    pub fn get_combat_option(&self, an: ActionName) -> Option<&CombatOption> {
        self.combat_actions.get(&an)
    }

    pub fn has_combat_option(&self, an: ActionName) -> bool {
        self.combat_actions.contains_key(&an)
    }

    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }
    pub fn get_resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    pub fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
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

    pub fn get_test_fighter_lvl0() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        Character::new(name, ability_scores, equipment)
    }

    pub fn get_test_fighter() -> Character {
        let mut fighter = get_test_fighter_lvl0();
        fighter.level_up(ClassName::Fighter, vec!()).unwrap();
        fighter
    }

    #[test]
    fn basic_character_test() {
        let fighter = get_test_fighter();
        assert_eq!("FighterMan", fighter.get_name());
        assert_eq!(3, fighter.get_ability_scores().strength.get_mod());
        assert!(fighter.get_ability_scores().strength.is_prof_save());
        assert_eq!(2, fighter.get_prof_bonus());
        assert_eq!("Greatsword", fighter.get_equipment().get_primary_weapon().get_name());
        assert_eq!(16, fighter.get_ac());
        assert_eq!(13, fighter.get_max_hp());
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
