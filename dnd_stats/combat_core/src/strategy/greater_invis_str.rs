use crate::actions::ActionName;
use crate::combat_state::CombatState;
use crate::conditions::ConditionName;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::spells::{SpellName, SpellSlot};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

pub struct GreaterInvisStrBuilder;
impl StrategyBuilder for GreaterInvisStrBuilder {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = GreaterInvisStr {
            participants,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct GreaterInvisStr<'pm> {
    participants: &'pm Vec<TeamMember>,
    my_pid: ParticipantId,
}
impl<'pm> Strategy for GreaterInvisStr<'pm> {
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        let has_slot = my_rm.get_current(ResourceName::SS(SpellSlot::Fourth)) > 0;
        let cm = state.get_cm(me);
        let invis = cm.has_condition(&ConditionName::Invisible);
        let cast_ba_spell = cm.has_condition(&ConditionName::CastBASpell);
        let no_conc = !cm.has_condition(&ConditionName::Concentration);
        let has_action = my_rm.get_current(ResourceName::RAT(ResourceActionType::Action)) > 0;
        if !invis && !cast_ba_spell && no_conc && has_slot && has_action {
            return StrategicAction::spell(ActionName::CastSpell(SpellName::GreaterInvis), Some(SpellSlot::Fourth)).into();
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
