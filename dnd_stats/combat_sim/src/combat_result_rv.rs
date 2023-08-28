use std::collections::BTreeMap;

use combat_core::participant::ParticipantId;
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::RandVar;

use crate::combat_result_rv::prob_combat_result::ProbCombatResult;
use crate::combat_state_rv::CombatStateRV;

pub mod prob_combat_result;

#[derive(Debug, Clone)]
pub struct CombatResultRV<T: RVProb> {
    states: Vec<ProbCombatResult<T>>,
}

impl<T: RVProb> CombatResultRV<T> {
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn get_pcr(&self, i: usize) -> &ProbCombatResult<T> {
        &self.states.get(i).unwrap()
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
            let prob = self.get_pcr(i).get_prob();
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
}

impl<'pm, T: RVProb> From<CombatStateRV<'pm, T>> for CombatResultRV<T> {
    fn from(value: CombatStateRV<'pm, T>) -> Self {
        let mut states = Vec::new();
        for state in value.get_states() {
            states.push(state.clone().into())
        }
        Self {
            states
        }
    }
}
