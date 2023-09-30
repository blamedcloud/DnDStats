use crate::actions::ActionName;
use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceName};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

pub struct ActionSurgeStrBuilder;
impl StrategyBuilder for ActionSurgeStrBuilder {
    fn build_strategy<'pm>(&self, _: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = ActionSurgeStr {
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct ActionSurgeStr {
    my_pid: ParticipantId
}

impl Strategy for ActionSurgeStr {
    fn get_participants(&self) -> &Vec<TeamMember> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::AN(ActionName::ActionSurge)) > 0 {
            return ActionName::ActionSurge.into()
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
