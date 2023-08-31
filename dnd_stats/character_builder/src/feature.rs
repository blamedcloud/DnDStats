use combat_core::ability_scores::Ability;
use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};

use crate::{CBError, Character};

pub mod feats;
pub mod fighting_style;

pub trait Feature {
    fn apply(&self, character: &mut Character) -> Result<(), CBError>;
}

pub struct AbilityScoreIncrease(pub Ability, pub Ability);
impl Feature for AbilityScoreIncrease {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.ability_scores.get_score_mut(&self.0).increase();
        character.ability_scores.get_score_mut(&self.1).increase();
        Ok(())
    }
}
impl From<Ability> for AbilityScoreIncrease {
    fn from(value: Ability) -> Self {
        AbilityScoreIncrease(value, value)
    }
}

pub struct SaveProficiencies {
    pub abilities: Vec<Ability>
}
impl Feature for SaveProficiencies {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        for ability in self.abilities.iter() {
            character.ability_scores.get_score_mut(ability).set_prof_save(true);
        }
        Ok(())
    }
}
impl From<Vec<Ability>> for SaveProficiencies {
    fn from(value: Vec<Ability>) -> Self {
        SaveProficiencies { abilities: value }
    }
}
impl From<Ability> for SaveProficiencies {
    fn from(value: Ability) -> Self {
        SaveProficiencies { abilities: vec!(value) }
    }
}
impl From<(Ability, Ability)> for SaveProficiencies {
    fn from(value: (Ability, Ability)) -> Self {
        SaveProficiencies { abilities: vec!(value.0, value.1)}
    }
}

pub struct ExtraAttack(pub u8);
impl Feature for ExtraAttack {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.combat_actions.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(self.0)));
        Ok(())
    }
}
