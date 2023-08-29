use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerResponse, TriggerType};

pub struct PairStrBuilder<S1, S2>
where
    S1: StrategyBuilder,
    S2: StrategyBuilder,
{
    str1: S1,
    str2: S2,
}

impl<S1, S2> PairStrBuilder<S1, S2>
where
    S1: StrategyBuilder,
    S2: StrategyBuilder,
{
    pub fn new(s1: S1, s2: S2) -> Self {
        Self {
            str1: s1,
            str2: s2,
        }
    }
}

impl<S1, S2> StrategyBuilder for PairStrBuilder<S1, S2>
where
    S1: StrategyBuilder,
    S2: StrategyBuilder,
{
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let strategies = vec!(
            self.str1.build_strategy(participants, me),
            self.str2.build_strategy(participants, me),
        );
        Box::new(LinearStrategy {
            strategies,
        })
    }
}

#[derive(Debug)]
pub struct LinearStrategy<'pm> {
    strategies: Vec<Box<dyn Strategy + 'pm>>,
}

impl<'pm> Strategy for LinearStrategy<'pm> {
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.strategies.first().unwrap().get_participants()
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.strategies.first().unwrap().get_my_pid()
    }

    fn get_action(&self, state: &CombatState) -> StrategyDecision {
        for str in self.strategies.iter() {
            let sd = str.get_action(state);
            if sd.is_some() {
                return sd;
            }
        }
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, tt: TriggerType, tc: TriggerContext, state: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        for str in self.strategies.iter() {
            let sub_v = str.handle_trigger(tt, tc, state);
            v.extend(sub_v.into_iter());
        }
        v
    }
}
