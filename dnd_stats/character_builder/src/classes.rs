use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use serde::{Deserialize, Serialize};

use combat_core::ability_scores::Ability;
use combat_core::health::HitDice;

use crate::{CBError, Character};
use crate::classes::fighter::FighterClass;
use crate::classes::ranger::VariantRangerClass;
use crate::classes::rogue::RogueClass;
use crate::classes::wizard::WizardClass;
use crate::feature::{Feature, SaveProficiencies};

pub mod fighter;
pub mod ranger;
pub mod rogue;
pub mod wizard;

pub trait Class {
    fn get_class_name(&self) -> ClassName;
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError>;
    fn get_subclass_features(&self, class_lvl: u8) -> Vec<Box<dyn Feature>> {
        vec!(Box::new(SubclassFeatures {
            class_name: self.get_class_name(),
            class_lvl,
        }))
    }
}

pub trait SubClass : Debug {
    fn get_class_name(&self) -> ClassName;
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError>;

    fn get_spellcasting_override(&self) -> Option<SpellCasterType> {
        None
    }
}

pub struct ChooseSubClass(pub Rc<dyn SubClass>);
impl Feature for ChooseSubClass {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.sub_classes.insert(self.0.get_class_name(),  self.0.clone());
        Ok(())
    }
}

pub struct SubclassFeatures {
    class_name: ClassName,
    class_lvl: u8,
}
impl Feature for SubclassFeatures {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let sub_class = character.get_sub_class(self.class_name).ok_or(CBError::NoSubClassSet)?;
        let features = sub_class.get_static_features(self.class_lvl)?;
        for feat in features {
            feat.apply(character)?;
        }
        Ok(())
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SpellCasterType {
    Martial,
    ThirdCaster,
    HalfCaster,
    FullCaster
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum ClassName {
    Barbarian,
    Bard,
    Cleric,
    Druid,
    Fighter,
    Monk,
    Paladin,
    Ranger,
    Rogue,
    Sorcerer,
    Warlock,
    Wizard,
}

impl ClassName {
    pub fn get_class(&self) -> Result<Box<dyn Class>, CBError> {
        match self {
            ClassName::Fighter => Ok(Box::new(FighterClass)),
            ClassName::Ranger => Ok(Box::new(VariantRangerClass)),
            ClassName::Rogue => Ok(Box::new(RogueClass)),
            ClassName::Wizard => Ok(Box::new(WizardClass)),
            _ => Err(CBError::NotImplemented)
        }
    }

    pub fn get_hit_die(&self) -> HitDice {
        match self {
            ClassName::Barbarian => HitDice::D12,
            ClassName::Bard => HitDice::D8,
            ClassName::Cleric => HitDice::D8,
            ClassName::Druid => HitDice::D8,
            ClassName::Fighter => HitDice::D10,
            ClassName::Monk => HitDice::D8,
            ClassName::Paladin => HitDice::D10,
            ClassName::Ranger => HitDice::D10,
            ClassName::Rogue => HitDice::D8,
            ClassName::Sorcerer => HitDice::D6,
            ClassName::Warlock => HitDice::D8,
            ClassName::Wizard => HitDice::D6,
        }
    }

    pub fn get_save_profs(&self) -> SaveProficiencies {
        let profs = match self {
            ClassName::Barbarian => (Ability::STR, Ability::CON),
            ClassName::Bard => (Ability::DEX, Ability::CHA),
            ClassName::Cleric => (Ability::WIS, Ability::CHA),
            ClassName::Druid => (Ability::INT, Ability::WIS),
            ClassName::Fighter => (Ability::STR, Ability::CON),
            ClassName::Monk => (Ability::STR, Ability::DEX),
            ClassName::Paladin => (Ability::WIS, Ability::CHA),
            ClassName::Ranger => (Ability::STR, Ability::DEX),
            ClassName::Rogue => (Ability::DEX, Ability::INT),
            ClassName::Sorcerer => (Ability::CON, Ability::CHA),
            ClassName::Warlock => (Ability::WIS, Ability::CHA),
            ClassName::Wizard => (Ability::INT, Ability::WIS),
        };
        profs.into()
    }

    pub fn get_default_spellcasting(&self) -> SpellCasterType {
        match self {
            ClassName::Barbarian => SpellCasterType::Martial,
            ClassName::Bard => SpellCasterType::FullCaster,
            ClassName::Cleric => SpellCasterType::FullCaster,
            ClassName::Druid => SpellCasterType::FullCaster,
            ClassName::Fighter => SpellCasterType::Martial,
            ClassName::Monk => SpellCasterType::Martial,
            ClassName::Paladin => SpellCasterType::HalfCaster,
            ClassName::Ranger => SpellCasterType::HalfCaster,
            ClassName::Rogue => SpellCasterType::Martial,
            ClassName::Sorcerer => SpellCasterType::FullCaster,
            ClassName::Warlock => SpellCasterType::FullCaster,
            ClassName::Wizard => SpellCasterType::FullCaster,
        }
    }
}

impl Display for ClassName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        match self {
            ClassName::Barbarian => s.push_str("Barbarian"),
            ClassName::Bard => s.push_str("Bard"),
            ClassName::Cleric => s.push_str("Cleric"),
            ClassName::Druid => s.push_str("Druid"),
            ClassName::Fighter => s.push_str("Fighter"),
            ClassName::Monk => s.push_str("Monk"),
            ClassName::Paladin => s.push_str("Paladin"),
            ClassName::Ranger => s.push_str("Ranger"),
            ClassName::Rogue => s.push_str("Rogue"),
            ClassName::Sorcerer => s.push_str("Sorcerer"),
            ClassName::Warlock => s.push_str("Warlock"),
            ClassName::Wizard => s.push_str("Wizard"),
        }
        write!(f, "{}", s)
    }
}
