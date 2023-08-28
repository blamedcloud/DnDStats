use std::collections::BTreeMap;
use combat_core::combat_event::CombatEvent;
use combat_core::participant::{ParticipantId, ParticipantManager};
use combat_core::transposition::Transposition;
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::RandVar;
use crate::combat_state_rv::prob_combat_state::ProbCombatState;

pub mod prob_combat_state;

#[derive(Debug, Clone)]
pub struct CombatStateRV<'pm, T: RVProb> {
    states: Vec<ProbCombatState<'pm, T>>,
}

impl<'pm, T: RVProb> CombatStateRV<'pm, T> {
    pub fn new(pm: &'pm ParticipantManager<T>) -> Self {
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

    pub fn get_pcs(&self, i: usize) -> &ProbCombatState<'pm, T> {
        &self.states.get(i).unwrap()
    }
    pub fn get_pcs_mut(&mut self, i: usize) -> &mut ProbCombatState<'pm, T> {
        self.states.get_mut(i).unwrap()
    }

    pub fn get_states(&self) -> &Vec<ProbCombatState<'pm, T>> {
        &self.states
    }
    pub fn get_states_mut(&mut self) -> &mut Vec<ProbCombatState<'pm, T>> {
        &mut self.states
    }

    pub fn get_index_rv(&self) -> RandomVariable<T> {
        let v: Vec<T> = self.states.iter().map(|pcs| pcs.get_prob()).cloned().collect();
        let ub = (self.len() as isize) - 1;
        RandomVariable::new(0, ub, v).unwrap()
    }

    pub fn get_dmg(&self, target: ParticipantId) -> RandomVariable<T> {
        let dmg_rvs = self.states.iter().map(|pcs| pcs.get_dmg(target));
        let mut pdf_map: BTreeMap<isize, T> = BTreeMap::new();
        for (i, rv) in dmg_rvs.enumerate() {
            let prob = self.get_pcs(i).get_prob();
            for dmg in rv.valid_p() {
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

impl<'pm, T: RVProb> From<Vec<ProbCombatState<'pm, T>>> for CombatStateRV<'pm, T> {
    fn from(value: Vec<ProbCombatState<'pm, T>>) -> Self {
        Self {
            states: value,
        }
    }
}
