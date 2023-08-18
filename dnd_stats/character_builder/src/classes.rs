use crate::ability_scores::Ability;
use crate::feature::{Feature, SaveProficiencies};
use crate::{CBError, Character, HitDice};
use crate::classes::fighter::FighterClass;

pub mod fighter;

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

pub trait SubClass : Feature {
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError>;
}

pub struct SubclassFeatures {
    class_name: ClassName,
    class_lvl: u8,
}
impl Feature for SubclassFeatures {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let sub_class = character.get_sub_class(self.class_name)?;
        let features = sub_class.get_static_features(self.class_lvl)?;
        for feat in features {
            feat.apply(character)?;
        }
        Ok(())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
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
}
