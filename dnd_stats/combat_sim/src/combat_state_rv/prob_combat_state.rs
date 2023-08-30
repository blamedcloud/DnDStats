use std::collections::{BTreeMap, HashMap};
use std::ptr;

use combat_core::actions::{ActionName, ActionType};
use combat_core::combat_event::{CombatEvent, CombatTiming};
use combat_core::combat_state::CombatState;
use combat_core::conditions::{ConditionManager, ConditionName};
use combat_core::health::Health;
use combat_core::participant::{ParticipantId, ParticipantManager};
use combat_core::resources::{RefreshTiming, ResourceManager, ResourceName};
use combat_core::resources::resource_amounts::ResourceCount;
use combat_core::transposition::Transposition;
use rand_var::map_rand_var::MapRandVar;
use rand_var::num_rand_var::NumRandVar;
use rand_var::rand_var::RandVar;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::rand_var::rv_partition::RVPartition;
use rand_var::vec_rand_var::VecRandVar;

use crate::combat_result_rv::prob_combat_result::ProbCombatResult;

#[derive(Debug, Clone)]
pub struct ProbCombatState<'pm, P: RVProb> {
    participants: &'pm ParticipantManager,
    state: CombatState,
    dmg: Vec<VecRandVar<P>>,
    prob: P,
}

impl<'pm, P: RVProb> ProbCombatState<'pm, P> {
    pub fn new(pm: &'pm ParticipantManager) -> Self {
        let mut dmg = Vec::with_capacity(pm.len());
        for _ in 0..pm.len() {
            dmg.push(VecRandVar::new_constant(0).unwrap());
        }
        Self {
            participants: pm,
            state: CombatState::new(pm.get_initial_rms(), pm.get_initial_cms()),
            dmg,
            prob: P::one(),
        }
    }

    pub fn push(&mut self, ce: CombatEvent) {
        self.state.push(ce);
    }

    pub fn get_state(&self) -> &CombatState {
        &self.state
    }

    pub fn get_last_event(&self) -> Option<CombatEvent> {
        self.state.get_last_event()
    }

    pub fn get_prob(&self) -> &P {
        &self.prob
    }

    pub fn handle_refresh(&mut self, pid: ParticipantId, rt: RefreshTiming) {
        if self.is_alive(pid) {
            self.get_rm_mut(pid).handle_timing(rt);
        }
    }

    pub fn get_rm(&self, pid: ParticipantId) -> &ResourceManager {
        self.state.get_rm(pid)
    }
    pub fn get_rm_mut(&mut self, pid: ParticipantId) -> &mut ResourceManager {
        self.state.get_rm_mut(pid)
    }

    pub fn get_cm(&self, pid: ParticipantId) -> &ConditionManager {
        self.state.get_cm(pid)
    }
    pub fn get_cm_mut(&mut self, pid: ParticipantId) -> &mut ConditionManager {
        self.state.get_cm_mut(pid)
    }

    pub fn get_max_hp(&self, pid: ParticipantId) -> isize {
        self.participants.get_participant(pid).participant.get_max_hp()
    }

    pub fn is_dead(&self, pid: ParticipantId) -> bool {
        self.state.is_dead(pid)
    }

    pub fn is_alive(&self, pid: ParticipantId) -> bool {
        self.state.is_alive(pid)
    }

    pub fn get_latest_timing(&self) -> Option<CombatTiming> {
        self.state.get_last_combat_timing()
    }

    pub fn is_valid_timing(&self, ct: CombatTiming) -> bool {
        let lt = self.get_latest_timing();
        if lt.is_some() && lt.unwrap() == CombatTiming::EncounterEnd {
            return false;
        }
        match ct {
            CombatTiming::EncounterBegin => lt.is_none(),
            CombatTiming::EncounterEnd => true,
            CombatTiming::BeginRound(_) => true,
            CombatTiming::EndRound(_) => true,
            CombatTiming::BeginTurn(pid) => self.is_alive(pid),
            CombatTiming::EndTurn(pid) => lt.unwrap() == CombatTiming::BeginTurn(pid)
        }
    }

    pub fn apply_default_condition(&mut self, pid: ParticipantId, cn: ConditionName) {
        let cm = self.get_cm_mut(pid);
        cm.add_basic_condition(cn).unwrap();
        self.push(CombatEvent::ApplyCond(cn, pid));
    }

    pub fn remove_condition(&mut self, pid: ParticipantId, cn: ConditionName, at: ActionType) {
        let cm = self.get_cm_mut(pid);
        cm.remove_condition(&cn);
        self.spend_at_resource(pid, at);
        self.push(CombatEvent::RemoveCond(cn));
    }

    pub fn get_dmg(&self, pid: ParticipantId) -> &VecRandVar<P> {
        self.dmg.get(pid.0).unwrap()
    }
    fn set_dmg(&mut self, pid: ParticipantId, rv: VecRandVar<P>) {
        self.dmg[pid.0] = rv;
    }

    pub fn spend_action_resources(&mut self, pid: ParticipantId, an: ActionName, at: ActionType) {
        let rm = self.get_rm_mut(pid);
        if rm.has_resource(ResourceName::AN(an)) {
            rm.spend(ResourceName::AN(an));
        }
        self.spend_at_resource(pid, at);
    }

    pub fn spend_at_resource(&mut self, pid: ParticipantId, at: ActionType) {
        let rm = self.get_rm_mut(pid);
        match at {
            ActionType::Movement => {
                if rm.has_resource(ResourceName::Movement) {
                    rm.drain(ResourceName::Movement);
                }
            }
            ActionType::HalfMove => {
                if rm.has_resource(ResourceName::Movement) {
                    let cap = rm.get_cap(ResourceName::Movement);
                    let amount: ResourceCount = (cap / 2).unwrap().into();
                    if !amount.is_uncapped() {
                        rm.spend_many(ResourceName::Movement, amount.count().unwrap());
                    }
                }
            }
            _ => {
                if rm.has_resource(at.into()) {
                    rm.spend(at.into());
                }
            }
        }
    }

    pub fn spend_resource_cost(&mut self, pid: ParticipantId, costs: HashMap<ResourceName, usize>) {
        let rm = self.get_rm_mut(pid);
        for (rn, cost) in costs {
            rm.spend_many(rn, cost);
        }
    }

    pub fn get_health(&self, pid: ParticipantId) -> Health {
        self.state.get_health(pid)
    }
    fn set_health(&mut self, pid: ParticipantId, h: Health) {
        self.state.set_health(pid, h);
    }

    pub fn split(self, rv: MapRandVar<CombatEvent, P>) -> Vec<Self> {
        let mut vec = Vec::with_capacity(rv.len());
        let child_state = self.state.into_child();
        for ce in rv.get_keys() {
            let mut ce_state = child_state.clone();
            ce_state.push(ce);
            vec.push(Self {
                participants: self.participants,
                state: ce_state,
                dmg: self.dmg.clone(),
                prob: self.prob.clone() * rv.pdf(ce)
            })
        }
        vec
    }

    pub fn split_dmg(self, state_rv: MapRandVar<CombatEvent, P>, dmg_map: BTreeMap<CombatEvent, VecRandVar<P>>, target: ParticipantId, dead_at_zero: bool) -> Vec<Self> {
        let children = self.split(state_rv);
        let mut result = Vec::with_capacity(children.len());
        for pcs in children.into_iter() {
            let ce = pcs.get_last_event().unwrap();
            result.extend(pcs.add_dmg(dmg_map.get(&ce).unwrap(), target, dead_at_zero).into_iter());
        }
        result
    }

    pub fn add_dmg(mut self, dmg: &VecRandVar<P>, target: ParticipantId, dead_at_zero: bool) -> Vec<Self> {
        let old_health = self.get_health(target);
        let hp = self.get_max_hp(target);
        let bloody_hp = Health::calc_bloodied(hp);
        let old_dmg = self.get_dmg(target);
        let new_dmg = old_dmg.add_rv(dmg).cap_lb(0).unwrap().cap_ub(hp).unwrap();

        let (new_hlb, new_hub) = Health::classify_bounds(&new_dmg, hp, dead_at_zero);
        let mut result = Vec::new();

        if new_hlb == new_hub {
            if new_hlb == old_health {
                self.set_dmg(target, new_dmg);
                result.push(self);
            } else {
                self.set_dmg(target, new_dmg);
                self.push(CombatEvent::HP(target, new_hlb));
                self.set_health(target, new_hlb);
                result.push(self);
            }
        } else {
            let child_state = self.state.clone().into_child();
            let partitions = new_dmg.partitions(|p| Health::classify_hp(p, bloody_hp, hp, dead_at_zero));
            for (new_health, partition) in partitions.into_iter() {
                let mut state = child_state.clone();
                if old_health != new_health {
                    state.push(CombatEvent::HP(target, new_health));
                    state.set_health(target, new_health);
                }
                let mut child = Self {
                    participants: self.participants,
                    state,
                    dmg: self.dmg.clone(),
                    prob: self.prob.clone() * partition.prob
                };
                child.set_dmg(target, partition.rv.unwrap());
                result.push(child);
            }
        }
        result
    }
}

impl<'pm, P: RVProb> Transposition for ProbCombatState<'pm, P> {
    fn is_transposition(&self, other: &Self) -> bool {
        let valid_state = CombatState::is_transposition(&self.state, &other.state);
        let valid_dmg = self.dmg.len() == other.dmg.len();
        let valid_prob = self.prob.clone() + other.prob.clone() <= P::one();
        let valid_part = ptr::eq(self.participants, other.participants);
        valid_state && valid_dmg && valid_prob && valid_part
    }

    fn merge_left(&mut self, mut other: Self) {
        let mut new_dmg = Vec::with_capacity(self.dmg.len());
        while self.dmg.len() > 0 {
            let left_dmg = self.dmg.pop().unwrap();
            let right_dmg = other.dmg.pop().unwrap();
            let left_part = RVPartition::new(self.prob.clone(), left_dmg);
            let right_part = RVPartition::new(other.prob.clone(), right_dmg);
            let new_part = left_part + right_part;
            new_dmg.push(new_part.rv.unwrap());
        }
        new_dmg.reverse();
        self.state.merge_left(other.state);
        self.dmg = new_dmg;
        self.prob = self.prob.clone() + other.prob;
    }
}

impl<'pm, P: RVProb> From<ProbCombatState<'pm, P>> for ProbCombatResult<P> {
    fn from(value: ProbCombatState<'pm, P>) -> Self {
        let mut part_data = Vec::new();
        for i in 0..value.participants.len() {
            part_data.push(value.participants.get_participant(ParticipantId(i)).into());
        }
        ProbCombatResult::new(part_data, value.state, value.dmg, value.prob)
    }
}
