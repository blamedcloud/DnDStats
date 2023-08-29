use rand_var::rv_traits::prob_type::RVProb;
use std::marker::PhantomData;
use crate::actions::ActionName;
use crate::combat_state::CombatState;
use crate::health::Health;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerResponse, TriggerType};

pub struct SecondWindStrBuilder;

impl<T: RVProb> StrategyBuilder<T> for SecondWindStrBuilder {
    fn build_strategy<'pm>(self, _: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = SecondWindStr {
            _t: PhantomData,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct SecondWindStr<T> {
    _t: PhantomData<T>,
    my_pid: ParticipantId
}

impl<T:RVProb> Strategy<T> for SecondWindStr<T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        let has_ba = my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0;
        let has_sw = my_rm.get_current(ResourceName::AN(ActionName::SecondWind)) > 0;
        if has_ba && has_sw && state.get_health(me) == Health::Bloodied {
            return ActionName::SecondWind.into()
        }
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
