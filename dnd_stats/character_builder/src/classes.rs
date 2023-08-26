use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::ability_scores::Ability;
use crate::feature::{Feature, SaveProficiencies};
use crate::{CBError, Character, HitDice};
use crate::classes::fighter::FighterClass;
use crate::classes::rogue::RogueClass;

pub mod fighter;
pub mod rogue;

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
}

pub struct ChooseSubClass<SC: SubClass + Clone>(pub SC);
impl<SC> Feature for ChooseSubClass<SC>
where
    SC: SubClass + Clone + 'static,
{
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.sub_classes.insert(self.0.get_class_name(),  Rc::new(self.0.clone()));
        Ok(())
    }
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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
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
            ClassName::Rogue => Ok(Box::new(RogueClass)),
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
