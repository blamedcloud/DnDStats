use std::rc::Rc;
use character_builder::combat::{ActionName, ActionType};
use character_builder::combat::attack::AttackResult;
use character_builder::resources::{RefreshTiming, ResourceManager, ResourceName};
use rand_var::rv_traits::prob_type::ProbType;
use crate::participant::ParticipantId;


#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct RoundId(pub u8);
impl From<u8> for RoundId {
    fn from(value: u8) -> Self {
        RoundId(value)
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum CombatTiming {
    BeginRound(RoundId),
    EndRound(RoundId),
    BeginTurn(ParticipantId),
    EndTurn(ParticipantId),
}
impl CombatTiming {
    pub fn get_refresh_timing(&self, pid: ParticipantId) -> RefreshTiming {
        match self {
            CombatTiming::BeginRound(_) => RefreshTiming::StartRound,
            CombatTiming::EndRound(_) => RefreshTiming::EndRound,
            CombatTiming::BeginTurn(t_pid) => {
                if pid == *t_pid {
                    RefreshTiming::StartMyTurn
                } else {
                    RefreshTiming::StartOtherTurn
                }
            }
            CombatTiming::EndTurn(t_pid) => {
                if pid == *t_pid {
                    RefreshTiming::EndMyTurn
                } else {
                    RefreshTiming::EndOtherTurn
                }
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum CombatEvent {
    Timing(CombatTiming),
    AN(ActionName),
    Attack(ParticipantId, ParticipantId),
    AR(AttackResult),
}

#[derive(Clone)]
pub struct CombatLog {
    parent: Option<CombatLogRef>,
    events: Vec<CombatEvent>,
}
type CombatLogRef = Rc<CombatLog>;

impl CombatLog {
    pub fn new() -> Self {
        Self {
            parent: None,
            events: Vec::new(),
        }
    }

    pub fn get_local_events(&self) -> &Vec<CombatEvent> {
        &self.events
    }

    pub fn get_all_events(&self) -> Vec<CombatEvent> {
        let mut all_events = Vec::new();
        if self.parent.is_some() {
            all_events = self.parent.as_ref().unwrap().get_all_events();
        }
        all_events.extend(self.events.iter());
        all_events
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
}

type ParticipantResources = Vec<ResourceManager>;
#[derive(Clone)]
pub struct CombatState {
    logs: CombatLog,
    resources: ParticipantResources,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            logs: CombatLog::new(),
            resources: ParticipantResources::new(),
        }
    }

    pub fn get_logs(&self) -> &CombatLog {
        &self.logs
    }
}

#[derive(Clone)]
pub struct ProbCombatState<T: ProbType> {
    state: CombatState,
    prob: T,
}

impl<T: ProbType> ProbCombatState<T> {
    pub fn new() -> Self {
        Self {
            state: CombatState::new(),
            prob: T::one(),
        }
    }

    pub fn add_participant(&mut self, rm: ResourceManager) {
        self.state.resources.push(rm);
    }

    pub fn push(&mut self, ce: CombatEvent) {
        self.state.logs.events.push(ce);
    }

    pub fn get_state(&self) -> &CombatState {
        &self.state
    }

    pub fn get_prob(&self) -> &T {
        &self.prob
    }

    pub fn get_rm(&self, pid: ParticipantId) -> &ResourceManager {
        &self.state.resources.get(pid.0).unwrap()
    }
    pub fn get_rm_mut(&mut self, pid: ParticipantId) -> &mut ResourceManager {
        self.state.resources.get_mut(pid.0).unwrap()
    }

    pub fn spend_resources(&mut self, pid: ParticipantId, an: ActionName, at: ActionType) {
        let rm = self.get_rm_mut(pid);
        if rm.has_resource(ResourceName::AN(an)) {
            rm.spend(ResourceName::AN(an));
        }
        if rm.has_resource(ResourceName::AT(at)) {
            rm.spend(ResourceName::AT(at));
        }
    }

}

pub struct CombatStateRV<T: ProbType> {
    states: Vec<ProbCombatState<T>>,
}

impl<T: ProbType> CombatStateRV<T> {
    pub fn new() -> Self {
        let mut states = Vec::new();
        states.push(ProbCombatState::new());
        Self {
            states,
        }
    }

    pub fn add_participant(&mut self, rm: ResourceManager) {
        for pcs in self.states.iter_mut() {
            pcs.state.resources.push(rm.clone());
        }
    }

    pub fn push(&mut self, ce: CombatEvent) {
        for pcs in self.states.iter_mut() {
            pcs.state.logs.events.push(ce);
        }
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn get_pcs(&self, i: usize) -> &ProbCombatState<T> {
        &self.states.get(i).unwrap()
    }
    pub fn get_pcs_mut(&mut self, i: usize) -> &mut ProbCombatState<T> {
        self.states.get_mut(i).unwrap()
    }

    pub fn get_states(&self) -> &Vec<ProbCombatState<T>> {
        &self.states
    }
    pub fn get_states_mut(&mut self) -> &mut Vec<ProbCombatState<T>> {
        &mut self.states
    }
}

impl<T: ProbType> From<Vec<ProbCombatState<T>>> for CombatStateRV<T> {
    fn from(value: Vec<ProbCombatState<T>>) -> Self {
        Self {
            states: value,
        }
    }
}
