use crate::actions::ActionName;
use crate::combat_state::CombatState;
use crate::health::Health;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

pub struct SecondWindStrBuilder;
impl StrategyBuilder for SecondWindStrBuilder {
    fn build_strategy<'pm>(&self, _: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = SecondWindStr {
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct SecondWindStr {
    my_pid: ParticipantId
}

impl Strategy for SecondWindStr {
    fn get_participants(&self) -> &Vec<TeamMember> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        let has_ba = my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0;
        let has_sw = my_rm.get_current(ResourceName::AN(ActionName::SecondWind)) > 0;
        if has_ba && has_sw && state.get_health(me) == Health::Bloodied {
            return ActionName::SecondWind.into()
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
