use std::rc::Rc;
use crate::{CBError, Character};
use crate::classes::{Class, ClassName, SubClass};
use crate::combat::{ActionName, ActionType, CombatAction, CombatOption};
use crate::feature::{ExtraAttack, Feature};

pub struct FighterClass;
impl Class for FighterClass {
    fn get_class_name(&self) -> ClassName {
        ClassName::Fighter
    }

    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            1 => Ok(vec!(Box::new(SecondWind))),
            2 => Ok(vec!(Box::new(ActionSurge))),
            3 => Ok(self.get_subclass_features(level)),
            4 => Ok(Vec::new()),
            5 => Ok(vec!(Box::new(ExtraAttack(2)))),
            6 => Ok(Vec::new()),
            7 => Ok(self.get_subclass_features(level)),
            8 => Ok(Vec::new()),
            _ => Err(CBError::NotImplemented)
        }
    }
}

pub struct ChampionFighter;
impl Feature for ChampionFighter {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.sub_classes.insert(ClassName::Fighter, Rc::new(ChampionFighter));
        Ok(())
    }
}
impl SubClass for ChampionFighter {
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            3 => Ok(vec!(Box::new(ImprovedCritical(19)))),
            7 => Ok(Vec::new()),
            10 => Ok(Vec::new()),
            15 => Ok(vec!(Box::new(ImprovedCritical(18)))),
            18 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel),
        }
    }
}

pub struct SecondWind;
impl Feature for SecondWind {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.combat_actions.insert(ActionName::SecondWind, CombatOption::new(ActionType::BonusAction, CombatAction::ByName));
        Ok(())
    }
}

pub struct ActionSurge;
impl Feature for ActionSurge {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.combat_actions.insert(ActionName::ActionSurge, CombatOption::new(ActionType::FreeAction, CombatAction::ByName));
        Ok(())
    }
}

pub struct ImprovedCritical(pub isize);
impl Feature for ImprovedCritical {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        for (_, co) in character.combat_actions.iter_mut() {
            if let CombatAction::Attack(wa) = &mut co.action {
                wa.set_crit_lb(self.0);
            }
        }
        Ok(())
    }
}
