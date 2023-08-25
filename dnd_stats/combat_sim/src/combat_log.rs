use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use character_builder::combat::{ActionName, ActionType};
use character_builder::combat::attack::AttackResult;
use character_builder::resources::{RefreshTiming, ResourceManager, ResourceName};
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::{NumRandVar, RandVar};
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
    EncounterBegin,
    EncounterEnd,
    BeginRound(RoundId),
    EndRound(RoundId),
    BeginTurn(ParticipantId),
    EndTurn(ParticipantId),
}

impl CombatTiming {
    pub fn get_refresh_timing(&self, pid: ParticipantId) -> Option<RefreshTiming> {
        match self {
            CombatTiming::EncounterBegin => None,
            CombatTiming::EncounterEnd => None,
            CombatTiming::BeginRound(_) => Some(RefreshTiming::StartRound),
            CombatTiming::EndRound(_) => Some(RefreshTiming::EndRound),
            CombatTiming::BeginTurn(t_pid) => {
                if pid == *t_pid {
                    Some(RefreshTiming::StartMyTurn)
                } else {
                    Some(RefreshTiming::StartOtherTurn)
                }
            },
            CombatTiming::EndTurn(t_pid) => {
                if pid == *t_pid {
                    Some(RefreshTiming::EndMyTurn)
                } else {
                    Some(RefreshTiming::EndOtherTurn)
                }
            },
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum CombatEvent {
    Timing(CombatTiming),
    AN(ActionName),
    Attack(ParticipantId, ParticipantId),
    AR(AttackResult),
    HP(ParticipantId, Health)
}

impl From<AttackResult> for CombatEvent {
    fn from(value: AttackResult) -> Self {
        CombatEvent::AR(value)
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum Health {
    Healthy,
    Bloodied,
    ZeroHP,
    Dead,
}

impl Health {
    pub fn classify_bounds<T: RVProb>(dmg: &RandomVariable<T>, max_hp: isize, dead_at_zero: bool) -> (Health, Health) {
        let bloodied = Health::calc_bloodied(max_hp);
        let lb = dmg.lower_bound();
        let ub = dmg.upper_bound();
        let lb_h = Health::classify_hp(&lb, bloodied, max_hp, dead_at_zero);
        let ub_h = Health::classify_hp(&ub, bloodied, max_hp, dead_at_zero);
        (lb_h, ub_h)
    }

    pub fn calc_bloodied(max_hp: isize) -> isize {
        if max_hp % 2 == 0 {
            max_hp / 2
        } else {
            (max_hp / 2) + 1
        }
    }

    pub fn classify_hp(dmg: &isize, bloody_hp: isize, max_hp: isize, dead_at_zero: bool) -> Health {
        if dmg < &bloody_hp {
            Health::Healthy
        } else if dmg < &max_hp {
            Health::Bloodied
        } else {
            if dead_at_zero {
                Health::Dead
            } else {
                Health::ZeroHP
            }
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn push(&mut self, ce: CombatEvent) {
        self.events.push(ce);
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

    pub fn get_last_event(&self) -> Option<CombatEvent> {
        if self.events.len() > 0 {
            self.events.last().copied()
        } else {
            if self.has_parent() {
                self.get_parent().get_last_event()
            } else {
                None
            }
        }
    }

    pub fn into_child(self) -> Self {
        Self {
            parent: Some(Rc::new(self)),
            events: Vec::new(),
        }
    }
}

type ParticipantResources = Vec<ResourceManager>;
type ParticipantHealth = Vec<Health>;

#[derive(Debug, Clone)]
pub struct CombatState {
    logs: CombatLog,
    resources: ParticipantResources,
    health: ParticipantHealth,
    deaths: HashSet<ParticipantId>,
    last_combat_timing: Option<CombatTiming>,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            logs: CombatLog::new(),
            resources: ParticipantResources::new(),
            health: ParticipantHealth::new(),
            deaths: HashSet::new(),
            last_combat_timing: None,
        }
    }

    pub fn add_participant(&mut self, rm: ResourceManager) {
        self.resources.push(rm);
        self.health.push(Health::Healthy); // TODO: full health?
    }

    pub fn get_logs(&self) -> &CombatLog {
        &self.logs
    }

    pub fn get_last_event(&self) -> Option<CombatEvent> {
        self.logs.get_last_event()
    }

    pub fn get_rm(&self, pid: ParticipantId) -> &ResourceManager {
        self.resources.get(pid.0).unwrap()
    }

    pub fn get_health(&self, pid: ParticipantId) -> Health {
        *self.health.get(pid.0).unwrap()
    }

    fn set_health(&mut self, pid: ParticipantId, h: Health) {
        self.health[pid.0] = h;
        if h == Health::Dead {
            self.deaths.insert(pid);
        }
    }

    pub fn into_child(self) -> Self {
        Self {
            logs: self.logs.into_child(),
            resources: self.resources,
            health: self.health,
            deaths: self.deaths,
            last_combat_timing: self.last_combat_timing,
        }
    }

    pub fn is_dead(&self, pid: ParticipantId) -> bool {
        self.deaths.contains(&pid)
    }

    pub fn is_alive(&self, pid: ParticipantId) -> bool {
        !self.is_dead(pid)
    }

    pub fn get_last_combat_timing(&self) -> Option<CombatTiming> {
        self.last_combat_timing
    }

    pub fn push(&mut self, ce: CombatEvent) {
        if let CombatEvent::Timing(ct) = ce {
            self.last_combat_timing = Some(ct);
        }
        self.logs.push(ce);
    }
}

type ParticipantMaxHP = Vec<isize>;
type ParticipantDmg<T> = Vec<RandomVariable<T>>;

#[derive(Debug, Clone)]
pub struct ProbCombatState<T: RVProb> {
    state: CombatState,
    max_hp: ParticipantMaxHP,
    dmg: ParticipantDmg<T>,
    prob: T,
}

impl<T: RVProb> ProbCombatState<T> {
    pub fn new() -> Self {
        Self {
            state: CombatState::new(),
            max_hp: ParticipantMaxHP::new(),
            dmg: ParticipantDmg::new(),
            prob: T::one(),
        }
    }

    pub fn add_participant(&mut self, rm: ResourceManager, hp: isize) {
        self.state.add_participant(rm);
        self.max_hp.push(hp);
        self.dmg.push(RandomVariable::new_constant(0).unwrap());
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

    pub fn get_prob(&self) -> &T {
        &self.prob
    }

    pub fn handle_refresh(&mut self, pid: ParticipantId, rt: RefreshTiming) {
        if self.is_alive(pid) {
            self.get_rm_mut(pid).handle_timing(rt);
        }
    }

    pub fn get_rm(&self, pid: ParticipantId) -> &ResourceManager {
        &self.state.resources.get(pid.0).unwrap()
    }
    pub fn get_rm_mut(&mut self, pid: ParticipantId) -> &mut ResourceManager {
        self.state.resources.get_mut(pid.0).unwrap()
    }

    pub fn get_hp(&self, pid: ParticipantId) -> isize {
        *self.max_hp.get(pid.0).unwrap()
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

    pub fn get_dmg(&self, pid: ParticipantId) -> &RandomVariable<T> {
        self.dmg.get(pid.0).unwrap()
    }
    fn set_dmg(&mut self, pid: ParticipantId, rv: RandomVariable<T>) {
        self.dmg[pid.0] = rv;
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
        let mut vec = Vec::with_capacity(rv.len());
        let child_state = self.state.into_child();
        for ce in rv.valid_p() {
            let mut ce_state = child_state.clone();
            ce_state.push(ce);
            vec.push(Self {
                state: ce_state,
                max_hp: self.max_hp.clone(),
                dmg: self.dmg.clone(),
                prob: self.prob.clone() * rv.pdf(ce)
            })
        }
        vec
    }

    pub fn split_dmg(self, state_rv: MapRandVar<CombatEvent, T>, dmg_map: BTreeMap<CombatEvent, RandomVariable<T>>, target: ParticipantId, dead_at_zero: bool) -> Vec<Self> {
        let children = self.split(state_rv);
        let mut result = Vec::with_capacity(children.len());
        for pcs in children.into_iter() {
            let ce = pcs.get_last_event().unwrap();
            result.extend(pcs.add_dmg(dmg_map.get(&ce).unwrap(), target, dead_at_zero).into_iter());
        }
        result
    }

    fn set_health(&mut self, pid: ParticipantId, h: Health) {
        self.state.set_health(pid, h);
    }

    pub fn get_health(&self, pid: ParticipantId) -> Health {
        self.state.get_health(pid)
    }

    pub fn add_dmg(mut self, dmg: &RandomVariable<T>, target: ParticipantId, dead_at_zero: bool) -> Vec<Self> {
        let old_health = self.get_health(target);
        let hp = self.get_hp(target);
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
                    state,
                    max_hp: self.max_hp.clone(),
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

    pub fn add_participant(&mut self, rm: ResourceManager, hp: isize) {
        for pcs in self.states.iter_mut() {
            pcs.add_participant(rm.clone(), hp);
        }
    }

    pub fn push(&mut self, ce: CombatEvent) {
        for pcs in self.states.iter_mut() {
            pcs.state.push(ce);
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
}

impl<T: RVProb> From<Vec<ProbCombatState<T>>> for CombatStateRV<T> {
    fn from(value: Vec<ProbCombatState<T>>) -> Self {
        Self {
            states: value,
        }
    }
}
