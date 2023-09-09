use std::collections::BTreeMap;

use combat_core::combat_event::CombatEvent;
use combat_core::participant::{ParticipantId, ParticipantManager};
use combat_core::resources::ResourceName;
use combat_core::transposition::Transposition;
use rand_var::map_rand_var::MapRandVar;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::rand_var::RandVar;
use rand_var::vec_rand_var::VecRandVar;

use crate::combat_state_rv::prob_combat_state::ProbCombatState;
use crate::CSError;

pub mod prob_combat_state;

#[derive(Debug, Clone)]
pub struct CombatStateRV<'pm, P: RVProb> {
    states: Vec<ProbCombatState<'pm, P>>,
}

impl<'pm, P: RVProb> CombatStateRV<'pm, P> {
    pub fn new(pm: &'pm ParticipantManager) -> Self {
        let mut states = Vec::new();
        states.push(ProbCombatState::new(pm));
        Self {
            states,
        }
    }

    pub fn push(&mut self, ce: CombatEvent) {
        for pcs in self.states.iter_mut() {
            pcs.push(ce);
        }
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn get_pcs(&self, i: usize) -> &ProbCombatState<'pm, P> {
        &self.states.get(i).unwrap()
    }
    pub fn get_pcs_mut(&mut self, i: usize) -> &mut ProbCombatState<'pm, P> {
        self.states.get_mut(i).unwrap()
    }

    // TODO: make these return slices?
    pub fn get_states(&self) -> &Vec<ProbCombatState<'pm, P>> {
        &self.states
    }
    pub fn get_states_mut(&mut self) -> &mut Vec<ProbCombatState<'pm, P>> {
        &mut self.states
    }

    pub fn get_index_rv(&self) -> VecRandVar<P> {
        let v: Vec<P> = self.states.iter().map(|pcs| pcs.get_prob()).cloned().collect();
        let ub = (self.len() as isize) - 1;
        VecRandVar::new(0, ub, v).unwrap()
    }

    pub fn get_dmg(&self, target: ParticipantId) -> VecRandVar<P> {
        let dmg_rvs = self.states.iter().map(|pcs| pcs.get_dmg(target));
        let mut pdf_map: BTreeMap<isize, P> = BTreeMap::new();
        for (i, rv) in dmg_rvs.enumerate() {
            let prob = self.get_pcs(i).get_prob();
            for dmg in rv.get_keys() {
                let dmg_prob = prob.clone() * rv.pdf(dmg);
                if pdf_map.contains_key(&dmg) {
                    let old_prob = pdf_map.get(&dmg).unwrap().clone();
                    pdf_map.insert(dmg, old_prob + dmg_prob);
                } else {
                    pdf_map.insert(dmg, dmg_prob);
                }
            }
        }
        MapRandVar::from_map(pdf_map).unwrap().into_rv()
    }

    pub fn get_resource_rv(&self, pid: ParticipantId, rn: ResourceName) -> Result<MapRandVar<isize, P>, CSError> {
        let mut pdf_map: BTreeMap<isize, P> = BTreeMap::new();
        let rms = self.states.iter().map(|pcs| pcs.get_rm(pid));
        for (i, rm) in rms.enumerate() {
            let prob = self.get_pcs(i).get_prob().clone();
            if rm.has_resource(rn) {
                let rc = rm.get_current(rn);
                if rc.is_uncapped() {
                    return Err(CSError::UncappedResource);
                } else {
                    let count = rc.count().unwrap() as isize;
                    if pdf_map.contains_key(&count) {
                        let old_prob = pdf_map.get(&count).unwrap().clone();
                        pdf_map.insert(count, old_prob + prob);
                    } else {
                        pdf_map.insert(count, prob);
                    }
                }
            }
        }
        Ok(MapRandVar::from_map(pdf_map)?)
    }

    pub fn merge_states(&mut self) {
        let mut new_states = Vec::with_capacity(self.len());
        while self.states.len() > 0 {
            let pcs2 = self.states.pop().unwrap();
            let mut is_merged = false;
            for i in 0..self.states.len() {
                if ProbCombatState::is_transposition(self.states.get(i).unwrap(), &pcs2) {
                    self.states[i].merge_left(pcs2.clone());
                    is_merged = true;
                    break;
                }
            }
            if !is_merged {
                new_states.push(pcs2);
            }
        }
        new_states.reverse();
        self.states = new_states;
    }
}

impl<'pm, P: RVProb> From<Vec<ProbCombatState<'pm, P>>> for CombatStateRV<'pm, P> {
    fn from(value: Vec<ProbCombatState<'pm, P>>) -> Self {
        Self {
            states: value,
        }
    }
}
