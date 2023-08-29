use std::marker::PhantomData;

use rand_var::rv_traits::prob_type::RVProb;

use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerResponse, TriggerType};

pub struct PairStrBuilder<T, S1, S2>
where
    T: RVProb,
    S1: StrategyBuilder<T>,
    S2: StrategyBuilder<T>,
{
    _t: PhantomData<T>,
    str1: S1,
    str2: S2,
}

impl<T, S1, S2> PairStrBuilder<T, S1, S2>
where
    T: RVProb,
    S1: StrategyBuilder<T>,
    S2: StrategyBuilder<T>,
{
    pub fn new(s1: S1, s2: S2) -> Self {
        Self {
            _t: PhantomData,
            str1: s1,
            str2: s2,
        }
    }
}

impl<T, S1, S2> StrategyBuilder<T> for PairStrBuilder<T, S1, S2>
where
    T: RVProb,
    S1: StrategyBuilder<T>,
    S2: StrategyBuilder<T>,
{
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
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
pub struct LinearStrategy<'pm, T: RVProb> {
    strategies: Vec<Box<dyn Strategy<T> + 'pm>>,
}

impl<'pm, T: RVProb> Strategy<T> for LinearStrategy<'pm, T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
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
