use std::fmt::Debug;

use rand_var::rv_traits::prob_type::RVProb;

use crate::actions::ActionName;
use crate::CCError;
use crate::combat_state::CombatState;
use crate::movement::Square;
use crate::participant::{Participant, ParticipantId, ParticipantManager, TeamMember};
use crate::triggers::{TriggerContext, TriggerResponse, TriggerType};

pub mod strategy_impls;

// TODO: add shapes eventually (for spells and such)
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum Target {
    Participant(ParticipantId),
    Tile(Square, Square),
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct StrategicOption {
    pub action_name: ActionName,
    pub target: Option<Target>
}

impl From<ActionName> for StrategicOption {
    fn from(value: ActionName) -> Self {
        Self {
            action_name: value,
            target: None,
        }
    }
}

pub trait StrategyBuilder<T: RVProb> {
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm>;
}

pub trait Strategy<T: RVProb> : Debug {
    fn get_participants(&self) -> &Vec<TeamMember<T>>;
    fn get_my_pid(&self) -> ParticipantId;

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption>;

    fn handle_trigger(&self, tt: TriggerType, tc: TriggerContext, state: &CombatState) -> Vec<TriggerResponse>;

    fn get_me(&self) -> &Box<dyn Participant<T>> {
        &self.get_participants().get(self.get_my_pid().0).unwrap().participant
    }

    fn get_first_target(&self, state: &CombatState) -> Option<Target> {
        let participants = self.get_participants();
        let me = self.get_my_pid();
        let my_team = participants.get(me.0).unwrap().team;
        for i in 0..participants.len() {
            let pid = ParticipantId(i);
            if participants[i].team != my_team && state.is_alive(pid) {
                return Some(Target::Participant(pid))
            }
        }
        None
    }
}

pub struct StrategyManager<'pm, T: RVProb> {
    pm: &'pm ParticipantManager<T>,
    strategies: Vec<Box<dyn Strategy<T> + 'pm>>,
    compiled: bool
}

impl<'pm, T: RVProb> StrategyManager<'pm, T> {
    pub fn new(pm: &'pm ParticipantManager<T>) -> Result<Self, CCError> {
        if !pm.is_compiled() {
            return Err(CCError::PMNotCompiled);
        }
        Ok(Self {
            pm,
            strategies: Vec::new(),
            compiled: false,
        })
    }

    pub fn add_participant(&mut self, str_bldr: impl StrategyBuilder<T>) -> Result<(), CCError> {
        if self.compiled {
            return Err(CCError::SMPushAfterCompile);
        }
        let str = str_bldr.build_strategy(self.pm.get_participants(), ParticipantId(self.strategies.len()));
        self.strategies.push(str);

        if self.len() == self.pm.len() {
            self.compile();
        }

        Ok(())
    }

    pub fn get_pm(&self) -> &'pm ParticipantManager<T> {
        self.pm
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    fn compile(&mut self) {
        self.compiled = true;
    }

    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    pub fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy<T> + 'pm> {
        self.strategies.get(pid.0).unwrap()
    }
}
