use crate::actions::ActionName;
use crate::combat_state::CombatState;
use crate::conditions::ConditionName;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::spells::{SpellName, SpellSlot};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

pub struct FireBallStrBuilder;
impl StrategyBuilder for FireBallStrBuilder {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = FireBallStr {
            participants,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct FireBallStr<'pm> {
    participants: &'pm Vec<TeamMember>,
    my_pid: ParticipantId,
}
impl<'pm> Strategy for FireBallStr<'pm> {
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        let has_slot = my_rm.get_current(ResourceName::SS(SpellSlot::Third)) > 0;
        let cast_ba_spell = state.get_cm(me).has_condition(&ConditionName::CastBASpell);
        if has_slot && !cast_ba_spell && my_rm.get_current(ResourceName::RAT(ResourceActionType::Action)) > 0 {
            let target = self.get_first_target(state);
            return StrategicAction::new(
                ActionName::CastSpell(SpellName::Fireball),
                target,
                Some(SpellSlot::Third)
            ).into();
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
