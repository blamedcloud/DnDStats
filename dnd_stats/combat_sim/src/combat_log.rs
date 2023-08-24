use std::rc::Rc;
use character_builder::combat::{ActionName, ActionType};
use character_builder::combat::attack::AttackResult;
use character_builder::resources::{RefreshTiming, ResourceManager, ResourceName};
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::RandVar;
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

impl From<AttackResult> for CombatEvent {
    fn from(value: AttackResult) -> Self {
        CombatEvent::AR(value)
    }
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

    pub fn get_parent(&self) -> &CombatLogRef {
        self.parent.as_ref().unwrap()
    }

    pub fn into_child(self) -> Self {
        Self {
            parent: Some(Rc::new(self)),
            events: Vec::new(),
        }
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

    pub fn get_rm(&self, pid: ParticipantId) -> &ResourceManager {
        self.resources.get(pid.0).unwrap()
    }

    pub fn into_child(self) -> Self {
        Self {
            logs: self.logs.into_child(),
            resources: self.resources
        }
    }

    pub fn push(&mut self, ce: CombatEvent) {
        self.logs.events.push(ce);
    }
}

#[derive(Clone)]
pub struct ProbCombatState<T: RVProb> {
    state: CombatState,
    prob: T,
}

impl<T: RVProb> ProbCombatState<T> {
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

    pub fn split(self, rv: MapRandVar<CombatEvent, T>) -> Vec<Self> {
        let mut v = Vec::with_capacity(rv.len());
        let child_state = self.state.into_child();
        for ce in rv.valid_p() {
            let mut ce_state = child_state.clone();
            ce_state.push(ce);
            v.push(Self {
                state: ce_state,
                prob: self.prob.clone() * rv.pdf(ce)
            })
        }
        v
    }

}

pub struct CombatStateRV<T: RVProb> {
    states: Vec<ProbCombatState<T>>,
}

impl<T: RVProb> CombatStateRV<T> {
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

    pub fn get_index_rv(&self) -> RandomVariable<T> {
        let v: Vec<T> = self.states.iter().map(|pcs| pcs.get_prob()).cloned().collect();
        let ub = (self.len() as isize) - 1;
        RandomVariable::new(0, ub, v).unwrap()
    }
}

impl<T: RVProb> From<Vec<ProbCombatState<T>>> for CombatStateRV<T> {
    fn from(value: Vec<ProbCombatState<T>>) -> Self {
        Self {
            states: value,
        }
    }
}
