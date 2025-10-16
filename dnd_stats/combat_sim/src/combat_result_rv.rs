use std::collections::BTreeMap;

use combat_core::participant::ParticipantId;
use combat_core::resources::ResourceName;
use rand_var::map_rand_var::MapRandVar;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::rand_var::RandVar;
use rand_var::vec_rand_var::VecRandVar;

use crate::combat_result_rv::prob_combat_result::ProbCombatResult;
use crate::combat_state_rv::CombatStateRV;
use crate::CSError;

pub mod prob_combat_result;

#[derive(Debug, Clone)]
pub struct CombatResultRV<P: RVProb> {
    states: Vec<ProbCombatResult<P>>,
}

impl<P: RVProb> CombatResultRV<P> {
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn get_pcr(&self, i: usize) -> &ProbCombatResult<P> {
        &self.states.get(i).unwrap()
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
            let prob = self.get_pcr(i).get_prob();
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
        MapRandVar::from_map(pdf_map).unwrap().into_vrv()
    }

    pub fn get_resource_rv(&self, pid: ParticipantId, rn: ResourceName) -> Result<MapRandVar<isize, P>, CSError> {
        let mut pdf_map: BTreeMap<isize, P> = BTreeMap::new();
        let rms = self.states.iter().map(|pcr| pcr.get_state().get_rm(pid));
        for (i, rm) in rms.enumerate() {
            let prob = self.get_pcr(i).get_prob().clone();
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
}

impl<'pm, P: RVProb> From<CombatStateRV<'pm, P>> for CombatResultRV<P> {
    fn from(value: CombatStateRV<'pm, P>) -> Self {
        let mut states = Vec::new();
        for state in value.get_states() {
            states.push(state.clone().into())
        }
        Self {
            states
        }
    }
}
