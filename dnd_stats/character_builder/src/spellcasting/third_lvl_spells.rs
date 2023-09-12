use combat_core::ability_scores::{Ability, ForceSave};
use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};
use combat_core::damage::{BasicDamageManager, DamageDice, DamageTerm, DamageType, ExtendedDamageDice, ExtendedDamageType};
use combat_core::damage::dice_expr::DiceExprTerm;
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
