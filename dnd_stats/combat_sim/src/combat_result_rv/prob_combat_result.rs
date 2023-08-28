use combat_core::combat_state::CombatState;
use combat_core::participant::{ParticipantData, ParticipantId};
use rand_var::RandomVariable;
use rand_var::rv_traits::prob_type::RVProb;

#[derive(Debug, Clone)]
pub struct ProbCombatResult<T: RVProb> {
    participants: Vec<ParticipantData>,
    state: CombatState,
    dmg: Vec<RandomVariable<T>>,
    prob: T,
}

impl<T: RVProb> ProbCombatResult<T> {
    pub fn new(participants: Vec<ParticipantData>, state: CombatState, dmg: Vec<RandomVariable<T>>, prob: T) -> Self {
        Self {
            participants,
            state,
            dmg,
            prob,
        }
    }

    pub fn get_participant_data(&self) -> &Vec<ParticipantData> {
        &self.participants
    }

    pub fn get_state(&self) -> &CombatState {
        &self.state
    }

    pub fn get_dmg(&self, pid: ParticipantId) -> &RandomVariable<T> {
        self.dmg.get(pid.0).unwrap()
    }

    pub fn get_prob(&self) -> &T {
        &self.prob
    }

    pub fn is_dead(&self, pid: ParticipantId) -> bool {
        self.state.is_dead(pid)
    }

    pub fn is_alive(&self, pid: ParticipantId) -> bool {
        self.state.is_alive(pid)
    }
}
