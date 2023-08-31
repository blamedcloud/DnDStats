use std::fmt::Debug;

use crate::actions::ActionType;
use crate::combat_state::CombatState;
use crate::conditions::ConditionLifetime;
use crate::participant::{ParticipantId, TeamMember};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

pub struct DoNothingBuilder;
impl StrategyBuilder for DoNothingBuilder {
    fn build_strategy<'pm>(&self, _: &'pm Vec<TeamMember>, _: ParticipantId) -> Box<dyn Strategy + 'pm> {
        Box::new(DoNothing)
    }
}

#[derive(Debug)]
pub struct DoNothing;
impl Strategy for DoNothing {
    fn get_participants(&self) -> &Vec<TeamMember> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        panic!("Should never call this!");
    }

    fn get_action(&self, _: &CombatState) -> StrategyDecision {
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}

pub struct RemoveCondBuilder;
impl StrategyBuilder for RemoveCondBuilder {
    fn build_strategy<'pm>(&self, _: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = RemoveConditions {
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct RemoveConditions {
    my_pid: ParticipantId,
}
impl Strategy for RemoveConditions {
    fn get_participants(&self) -> &Vec<TeamMember> {
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

    fn handle_trigger(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
