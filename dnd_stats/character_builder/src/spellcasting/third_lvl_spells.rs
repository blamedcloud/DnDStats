use combat_core::ability_scores::{Ability, ForceSave};
use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};
use combat_core::conditions::{Condition, ConditionEffect, ConditionLifetime, ConditionName};
use combat_core::D20RollType;
use combat_core::damage::{BasicDamageManager, DamageDice, DamageTerm, DamageType, ExtendedDamageDice, ExtendedDamageType};
use combat_core::damage::dice_expr::DiceExprTerm;
use combat_core::resources::resource_amounts::{RefreshBy, ResourceCap, ResourceCount};
use combat_core::resources::{RefreshTiming, Resource, ResourceActionType, ResourceName};
use combat_core::spells::{SaveDmgSpell, Spell, SpellEffect, SpellName, SpellSlot};
use crate::{CBError, Character};
use crate::feature::Feature;

// TODO spells should probably use a setup similar to combat actions, where they are built as
// char specific things, and then later transformed into basic saves and whatever.
pub struct FireBallSpell(pub Ability);
impl Feature for FireBallSpell {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let co = CombatOption::new_target(ActionType::Action, CombatAction::CastSpell, true);
        character.combat_actions.insert(ActionName::CastSpell(SpellName::Fireball), co);

        let save_dc = 8 + (character.get_prof_bonus() as isize) + (character.get_ability_scores().get_score(&self.0).get_mod() as isize);
        let save = ForceSave::new(Ability::DEX, save_dc);
        let mut dmg = BasicDamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(
            DiceExprTerm::Dice(8, ExtendedDamageDice::Basic(DamageDice::D6)),
            ExtendedDamageType::Basic(DamageType::Fire)
        ));

        let spell_effect = SpellEffect::SaveDamage(SaveDmgSpell::new(save, dmg, true));
        let spell = Spell::new(SpellSlot::Third, spell_effect);
        character.spell_manager.insert(SpellName::Fireball, spell);

        Ok(())
    }
}

pub struct HasteSpell;
impl Feature for HasteSpell {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let co = CombatOption::new(ActionType::Action, CombatAction::CastSpell);
        character.combat_actions.insert(ActionName::CastSpell(SpellName::Haste), co);

        // TODO: double speed somehow?
        let cond_effects = vec!(
            ConditionEffect::ACBonus(2),
            ConditionEffect::SaveMod(Ability::DEX, D20RollType::Advantage),
            ConditionEffect::ChangeResourceCap(ResourceName::AN(ActionName::HasteAction), ResourceCap::Hard(1)),
        );
        let cond = Condition {
            effects: cond_effects,
            lifetimes: vec!(ConditionLifetime::DropConcentration),
        };
        let spell_effect = SpellEffect::ApplyCondition(ConditionName::Hasted, cond);
        let spell = Spell::concentration(SpellSlot::Third, spell_effect, true);
        character.spell_manager.insert(SpellName::Haste, spell);

        let mut res = Resource::new(ResourceCap::Hard(0), ResourceCount::Count(0));
        res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToFull);
        res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToEmpty);
        character.resource_manager.add_perm(ResourceName::AN(ActionName::HasteAction), res);

        // TODO: eventually, I'll need to allow for the other haste actions
        character.combat_actions.insert(
            ActionName::HasteAction,
            CombatOption::new(
                ActionType::FreeAction,
                CombatAction::GainResource(ResourceName::RAT(ResourceActionType::SingleAttack), 1)
            )
        );

        // TODO: add triggers for drop conc (change resource cap and haste lethargy)

        Ok(())
    }
}
