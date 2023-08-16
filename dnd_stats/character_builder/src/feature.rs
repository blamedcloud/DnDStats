use crate::ability_scores::Ability;
use crate::Character;
use crate::combat::{ActionName, ActionType, CombatAction, CombatOption};

pub mod feats;
pub mod fighting_style;

pub trait Feature {
    fn apply(&self, character: &mut Character);
}


pub struct AbilityScoreIncrease(Ability, Ability);

impl Feature for AbilityScoreIncrease {
    fn apply(&self, character: &mut Character) {
        character.ability_scores.get_score_mut(&self.0).increase();
        character.ability_scores.get_score_mut(&self.1).increase();
    }
}

impl From<Ability> for AbilityScoreIncrease {
    fn from(value: Ability) -> Self {
        AbilityScoreIncrease(value, value)
    }
}

pub struct SaveProficiencies {
    abilities: Vec<Ability>
}

impl Feature for SaveProficiencies {
    fn apply(&self, character: &mut Character) {
        for ability in self.abilities.iter() {
            character.ability_scores.get_score_mut(ability).set_prof_save(true);
        }
    }
}

impl From<Vec<Ability>> for SaveProficiencies {
    fn from(value: Vec<Ability>) -> Self {
        SaveProficiencies { abilities: value }
    }
}

pub struct ExtraAttack(u8);

impl Feature for ExtraAttack {
    fn apply(&self, character: &mut Character) {
        character.combat_actions.insert(ActionName::AttackAction, CombatOption::new(ActionType::Action, CombatAction::AdditionalAttacks(self.0)));
    }
}

