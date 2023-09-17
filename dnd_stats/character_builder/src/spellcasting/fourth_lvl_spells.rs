use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};
use combat_core::conditions::{AttackDistance, Condition, ConditionEffect, ConditionLifetime, ConditionName};
use combat_core::D20RollType;
use combat_core::spells::{Spell, SpellEffect, SpellName, SpellSlot};
use crate::{CBError, Character};
use crate::feature::Feature;

pub struct GreaterInvisibilitySpell;
impl Feature for GreaterInvisibilitySpell {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let co = CombatOption::new(ActionType::Action, CombatAction::CastSpell);
        character.combat_actions.insert(ActionName::CastSpell(SpellName::GreaterInvis), co);

        let cond_effects = vec!(
            ConditionEffect::AttackerMod(AttackDistance::Any, D20RollType::Advantage),
            ConditionEffect::AtkTargetedMod(AttackDistance::Any, D20RollType::Disadvantage),
        );
        let cond = Condition {
            effects: cond_effects,
            lifetimes: vec!(ConditionLifetime::DropConcentration),
        };

        let spell_effect = SpellEffect::ApplyCondition(ConditionName::Invisible, cond);
        let spell = Spell::concentration(SpellSlot::Fourth, spell_effect, true);
        character.spell_manager.insert(SpellName::GreaterInvis, spell);

        Ok(())
    }
}
