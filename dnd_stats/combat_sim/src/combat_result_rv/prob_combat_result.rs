use combat_core::combat_state::CombatState;
use combat_core::participant::{ParticipantData, ParticipantId};
use rand_var::vec_rand_var::VecRandVar;
use rand_var::rand_var::prob_type::RVProb;

#[derive(Debug, Clone)]
pub struct ProbCombatResult<P: RVProb> {
    participants: Vec<ParticipantData>,
    state: CombatState,
    dmg: Vec<VecRandVar<P>>,
    prob: P,
}

impl<P: RVProb> ProbCombatResult<P> {
    pub fn new(participants: Vec<ParticipantData>, state: CombatState, dmg: Vec<VecRandVar<P>>, prob: P) -> Self {
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

    pub fn get_dmg(&self, pid: ParticipantId) -> &VecRandVar<P> {
        self.dmg.get(pid.0).unwrap()
    }

    pub fn get_prob(&self) -> &P {
        &self.prob
    }

    pub fn is_dead(&self, pid: ParticipantId) -> bool {
        self.state.is_dead(pid)
    }

    pub fn is_alive(&self, pid: ParticipantId) -> bool {
        self.state.is_alive(pid)
    }
}
