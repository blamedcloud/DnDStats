use std::fmt::Debug;
use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

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
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let strategies = vec!(
            self.str1.build_strategy(participants, me),
            self.str2.build_strategy(participants, me),
        );
        Box::new(LinearStrategy {
            strategies,
        })
    }
}

pub struct LinearStrategyBuilder {
    str_builders: Vec<Box<dyn StrategyBuilder>>,
}

impl LinearStrategyBuilder {
    pub fn new() -> Self {
        Self {
            str_builders: Vec::new(),
        }
    }

    pub fn add_str_bldr(&mut self, str_bldr: Box<dyn StrategyBuilder>) {
        self.str_builders.push(str_bldr);
    }
}

impl From<Vec<Box<dyn StrategyBuilder>>> for LinearStrategyBuilder {
    fn from(value: Vec<Box<dyn StrategyBuilder>>) -> Self {
        let mut lsb = LinearStrategyBuilder::new();
        for sb in value {
            lsb.add_str_bldr(sb);
        }
        lsb
    }
}

impl StrategyBuilder for LinearStrategyBuilder {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let mut strategies = Vec::new();
        for str_bldr in self.str_builders.iter() {
            strategies.push(str_bldr.build_strategy(participants, me));
        }
        Box::new(LinearStrategy {
            strategies
        })
    }
}

#[derive(Debug)]
pub struct LinearStrategy<'pm> {
    strategies: Vec<Box<dyn Strategy + 'pm>>,
}

impl<'pm> LinearStrategy<'pm> {
    pub fn new(strategies: Vec<Box<dyn Strategy + 'pm>>) -> Self {
        Self {
            strategies,
        }
    }
}

impl<'pm> Strategy for LinearStrategy<'pm> {
    // TODO: this and get_my_pid should be fixed in case on of them panics for the first strategy
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.strategies.first().unwrap().get_participants()
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.strategies.first().unwrap().get_my_pid()
    }

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        for str in self.strategies.iter() {
            let sd = str.choose_action(state);
            if sd.is_some() {
                return sd;
            }
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, ti: TriggerInfo, state: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        for str in self.strategies.iter() {
            let sub_v = str.choose_triggers(ti, state);
            v.extend(sub_v.into_iter());
        }
        v
    }
}
