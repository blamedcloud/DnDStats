use std::fmt::Debug;
use std::marker::PhantomData;

use rand_var::rv_traits::prob_type::RVProb;

use crate::actions::ActionType;
use crate::combat_state::CombatState;
use crate::conditions::ConditionLifetime;
use crate::participant::{ParticipantId, TeamMember};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerResponse, TriggerType};

pub struct DoNothingBuilder;
impl<T: RVProb> StrategyBuilder<T> for DoNothingBuilder {
    fn build_strategy<'pm>(self, _: &'pm Vec<TeamMember<T>>, _: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        Box::new(DoNothing)
    }
}

#[derive(Debug)]
pub struct DoNothing;
impl<T: RVProb> Strategy<T> for DoNothing {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        panic!("Should never call this!");
    }

    fn get_action(&self, _: &CombatState) -> StrategyDecision {
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}

pub struct RemoveCondBuilder;
impl<T: RVProb> StrategyBuilder<T> for RemoveCondBuilder {
    fn build_strategy<'pm>(self, _: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = RemoveConditions {
            _t: PhantomData,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct RemoveConditions<T> {
    _t: PhantomData<T>,
    my_pid: ParticipantId,
}
impl<T: RVProb> Strategy<T> for RemoveConditions<T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> StrategyDecision {
        let my_cm = state.get_cm(self.get_my_pid());
        for at in ActionType::iterator() {
            let lifetime = ConditionLifetime::UntilSpendAT(at);
            if my_cm.has_lifetime(&lifetime) {
                let cns = my_cm.get_cns_for_lifetime(&lifetime);
                return StrategyDecision::RemoveCondition(*cns.first().unwrap(), at);
            }
        }
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
