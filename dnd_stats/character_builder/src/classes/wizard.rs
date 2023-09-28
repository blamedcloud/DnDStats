use crate::CBError;
use crate::classes::{Class, ClassName, SubClass};
use crate::feature::Feature;

pub struct WizardClass;
impl Class for WizardClass {
    fn get_class_name(&self) -> ClassName {
        ClassName::Wizard
    }

    // TODO: impl features
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            1 => Ok(Vec::new()),
            2 => Ok(self.get_subclass_features(level)),
            3 => Ok(Vec::new()),
            4 => Ok(Vec::new()),
            5 => Ok(Vec::new()),
            6 => Ok(self.get_subclass_features(level)),
            7 => Ok(Vec::new()),
            8 => Ok(Vec::new()),
            9 => Ok(Vec::new()),
            10 => Ok(self.get_subclass_features(level)),
            11 => Ok(Vec::new()),
            12 => Ok(Vec::new()),
            13 => Ok(Vec::new()),
            14 => Ok(self.get_subclass_features(level)),
            15 => Ok(Vec::new()),
            16 => Ok(Vec::new()),
            17 => Ok(Vec::new()),
            18 => Ok(Vec::new()),
            19 => Ok(Vec::new()),
            20 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ConjurationWizard;
impl SubClass for ConjurationWizard {
    fn get_class_name(&self) -> ClassName {
        ClassName::Wizard
    }

    // TODO: impl features
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            2 => Ok(Vec::new()),
            6 => Ok(Vec::new()),
            10 => Ok(Vec::new()),
            14 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use combat_core::ability_scores::{Ability, AbilityScores};
    use crate::Character;
    use crate::classes::{ChooseSubClass, ClassName};
    use crate::classes::wizard::ConjurationWizard;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::AbilityScoreIncrease;

    #[test]
    fn lvl20_wizard() {
        let equipment = Equipment::new(
            Armor::no_armor(),
            Weapon::quarterstaff(),
            OffHand::Free
        );
        let scores = AbilityScores::new(8, 12, 14, 16, 13, 8);
        let mut wizard = Character::new(String::from("baelin"), scores, equipment);
        wizard.level_up(ClassName::Wizard, vec!()).unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(ChooseSubClass(Rc::new(ConjurationWizard))))).unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(AbilityScoreIncrease::from(Ability::INT)))).unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(AbilityScoreIncrease::from(Ability::INT)))).unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        wizard.level_up_basic().unwrap();
        assert_eq!(20, wizard.get_level());
        assert_eq!(20, wizard.ability_scores.intelligence.get_score());
        assert_eq!(20, wizard.ability_scores.constitution.get_score());
    }
}
