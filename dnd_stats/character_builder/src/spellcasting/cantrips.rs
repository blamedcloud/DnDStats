use combat_core::ability_scores::Ability;
use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};
use combat_core::attack::basic_attack::BasicAttack;
use combat_core::damage::{DamageDice, DamageManager, DamageTerm, DamageType, ExtendedDamageDice, ExtendedDamageType};
use combat_core::damage::dice_expr::DiceExprTerm;
use combat_core::spells::{Spell, SpellEffect, SpellName, SpellSlot};
use crate::{CBError, Character};
use crate::feature::Feature;

pub fn get_cantrip_dice(level: u8) -> u8 {
    let mut num_dice = 1;
    if level >= 17 {
        num_dice = 4;
    } else if level >= 11 {
        num_dice = 3;
    } else if level >= 5 {
        num_dice = 2;
    }
    num_dice
}

// TODO spells should probably use a setup similar to combat actions, where they are built as
// char specific things, and then later transformed into basic attacks and whatever.
pub struct FireBoltCantrip(pub Ability);
impl Feature for FireBoltCantrip {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let co = CombatOption::new_spell(ActionType::Action, CombatAction::CastSpell, true, true);
        character.combat_actions.insert(ActionName::CastSpell(SpellName::FireBolt), co);

        let num_die = get_cantrip_dice(character.get_level());
        let mut dmg = DamageManager::new();
        dmg.add_base_dmg(DamageTerm::new(
            DiceExprTerm::Dice(num_die, ExtendedDamageDice::Basic(DamageDice::D10)),
            ExtendedDamageType::Basic(DamageType::Fire)
        ));
        let bonus = (character.get_prof_bonus() as isize) + (character.get_ability_scores().get_score(&self.0).get_mod() as isize);
        let atk = BasicAttack::prebuilt(dmg, bonus, 20);
        let spell_effect = SpellEffect::SpellAttack(atk);
        let spell = Spell::new(SpellSlot::Cantrip, spell_effect);
        character.spell_manager.insert(SpellName::FireBolt, spell);

        Ok(())
    }
}
