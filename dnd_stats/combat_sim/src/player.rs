use std::collections::HashSet;
use std::fmt::Debug;

use character_builder::Character;
use character_builder::spellcasting::character_spell_slots;
use combat_core::ability_scores::AbilityScores;
use combat_core::actions::{ActionManager, CombatAction, CombatOption, register_pid};
use combat_core::attack::basic_attack::BasicAttack;
use combat_core::conditions::ConditionManager;
use combat_core::damage::DamageType;
use combat_core::damage::dice_expr::DiceExpression;
use combat_core::participant::{Participant, ParticipantId};
use combat_core::resources::ResourceManager;
use combat_core::skills::SkillManager;
use combat_core::spells::SpellManager;
use combat_core::triggers::TriggerManager;

#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    ac: isize,
    max_hp: isize,
    prof: isize,
    resistances: HashSet<DamageType>,
    ability_scores: AbilityScores,
    skill_manager: SkillManager,
    action_manager: ActionManager,
    resource_manager: ResourceManager,
    condition_manager: ConditionManager,
    trigger_manager: TriggerManager,
    spell_manager: SpellManager,
}

impl Player {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl From<Character> for Player {
    fn from(value: Character) -> Self {
        let mut am = ActionManager::new();

        for (an, co) in value.get_action_builder().clone() {
            let ca_old = co.action;
            let at = co.action_type;
            let req_t = co.req_target;
            let is_spell = co.is_spell;
            let ca: CombatAction<BasicAttack, DiceExpression> = match ca_old {
                CombatAction::Attack(wa) => CombatAction::Attack(wa.into()),
                CombatAction::SelfHeal(cde) => CombatAction::SelfHeal(cde.into()),
                CombatAction::GainResource(rn, aa) => CombatAction::GainResource(rn, aa),
                CombatAction::ApplyBasicCondition(cn) => CombatAction::ApplyBasicCondition(cn),
                CombatAction::ApplyComplexCondition(cn, cond) => CombatAction::ApplyComplexCondition(cn, cond),
                CombatAction::CastSpell => CombatAction::CastSpell,
                CombatAction::ByName => CombatAction::ByName
            };
            let co = CombatOption::new_spell(at, ca, req_t, is_spell);
            am.insert(an, co);
        }

        let mut rm = value.get_resource_manager().clone();
        rm.set_spell_slots(character_spell_slots(&value));

        Self {
            name: value.get_name().to_string(),
            ac: value.get_ac() as isize,
            max_hp: value.get_max_hp(),
            prof: value.get_prof_bonus() as isize,
            resistances: value.get_resistances().clone(),
            ability_scores: value.get_ability_scores().clone(),
            skill_manager: value.get_skills().clone(),
            action_manager: am,
            resource_manager: rm,
            trigger_manager: value.get_trigger_manager().clone(),
            condition_manager: value.get_condition_manager().clone(),
            spell_manager: value.get_spell_manager().clone(),
        }
    }
}

impl Participant for Player {
    fn get_ac(&self) -> isize {
        self.ac
    }

    fn get_max_hp(&self) -> isize {
        self.max_hp
    }

    fn get_prof(&self) -> isize {
        self.prof
    }

    fn get_resistances(&self) -> &HashSet<DamageType> {
        &self.resistances
    }

    fn get_ability_scores(&self) -> &AbilityScores {
        &self.ability_scores
    }

    fn get_skill_manager(&self) -> &SkillManager {
        &self.skill_manager
    }

    fn get_action_manager(&self) -> &ActionManager {
        &self.action_manager
    }

    fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    fn get_condition_manager(&self) -> &ConditionManager {
        &self.condition_manager
    }

    fn has_triggers(&self) -> bool {
        true
    }

    fn get_trigger_manager(&self) -> Option<&TriggerManager> {
        Some(&self.trigger_manager)
    }

    fn has_spells(&self) -> bool {
        true
    }

    fn get_spell_manager(&self) -> Option<&SpellManager> {
        Some(&self.spell_manager)
    }

    fn register_pid(&mut self, pid: ParticipantId) {
        register_pid(&mut self.action_manager, pid);
        self.trigger_manager.register_pid(pid);
    }
}
