use std::collections::{HashMap, HashSet};

use crate::attack::AttackResult;
use crate::conditions::{Condition, ConditionName};
use crate::damage::DamageTerm;
use crate::participant::ParticipantId;
use crate::resources::ResourceName;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerType {
    WasHit,
    SuccessfulAttack,
    OnKill,
    DropConc,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerContext {
    NoContext,
    AR(AttackResult),
    CondNotice(ConditionName),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TriggerInfo {
    pub tt: TriggerType,
    pub tc: TriggerContext,
}

impl TriggerInfo {
    pub fn new(tt: TriggerType, tc: TriggerContext) -> Self {
        Self {
            tt,
            tc,
        }
    }
}
impl From<TriggerType> for TriggerInfo {
    fn from(value: TriggerType) -> Self {
        Self {
            tt: value,
            tc: TriggerContext::NoContext,
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerName {
    SneakAttack,
    GWMBonusAtk,
    FavoredFoeKill,
    HasteLethargy,
}

#[derive(Debug, Clone)]
pub enum TriggerAction {
    IncreaseAC(isize),
    AddAttackDamage(DamageTerm),
    AddResource(ResourceName, usize),
    SetResourceLock(ResourceName, bool),
    GiveCondition(ConditionName, Condition),
}

#[derive(Debug, Clone)]
pub struct TriggerResponse {
    pub action: TriggerAction,
    pub resources: Vec<ResourceName>,
}

impl TriggerResponse {
    pub fn new(ta: TriggerAction, resources: Vec<ResourceName>) -> Self {
        Self {
            action: ta,
            resources,
        }
    }

    pub fn register_pid(&mut self, pid: ParticipantId) {
        match &mut self.action {
            TriggerAction::GiveCondition(_, cond) => {
                cond.register_pid(pid);
            },
            _ => {}
        }
    }
}
impl From<TriggerAction> for TriggerResponse {
    fn from(value: TriggerAction) -> Self {
        TriggerResponse::new(value, Vec::new())
    }
}
impl From<(TriggerAction, ResourceName)> for TriggerResponse {
    fn from(value: (TriggerAction, ResourceName)) -> Self {
        TriggerResponse::new(value.0, vec!(value.1))
    }
}

#[derive(Debug, Clone)]
pub struct TriggerManager {
    pub auto_triggers: HashMap<TriggerInfo, HashSet<TriggerName>>,
    pub manual_triggers: HashMap<TriggerInfo, HashSet<TriggerName>>,
    pub responses: HashMap<TriggerName, TriggerResponse>,
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            auto_triggers: HashMap::new(),
            manual_triggers: HashMap::new(),
            responses: HashMap::new(),
        }
    }

    pub fn register_pid(&mut self, pid: ParticipantId) {
        for (_, tr) in self.responses.iter_mut() {
            tr.register_pid(pid);
        }
    }

    pub fn add_auto_trigger(&mut self, ti: TriggerInfo, tn: TriggerName) {
        if self.auto_triggers.contains_key(&ti) {
            let mut oldv = self.auto_triggers.remove(&ti).unwrap();
            oldv.insert(tn);
            self.auto_triggers.insert(ti, oldv);
        } else {
            let mut hs = HashSet::new();
            hs.insert(tn);
            self.auto_triggers.insert(ti, hs);
        }
    }

    pub fn add_manual_trigger(&mut self, ti: TriggerInfo, tn: TriggerName) {
        if self.manual_triggers.contains_key(&ti) {
            let mut oldv = self.manual_triggers.remove(&ti).unwrap();
            oldv.insert(tn);
            self.manual_triggers.insert(ti, oldv);
        } else {
            let mut hs = HashSet::new();
            hs.insert(tn);
            self.manual_triggers.insert(ti, hs);
        }
    }

    pub fn add_manual_triggers(&mut self, ti: TriggerInfo, tn_hs: HashSet<TriggerName>) {
        if self.manual_triggers.contains_key(&ti) {
            let mut oldv = self.manual_triggers.remove(&ti).unwrap();
            oldv.extend(tn_hs.into_iter());
            self.manual_triggers.insert(ti, oldv);
        } else {
            self.manual_triggers.insert(ti, tn_hs);
        }
    }

    pub fn replace_manual_trigger(&mut self, ti: TriggerInfo, tn_hs: HashSet<TriggerName>) {
        self.manual_triggers.insert(ti, tn_hs);
    }

    pub fn get_auto_trigger_names(&self, ti: TriggerInfo) -> Option<&HashSet<TriggerName>> {
        self.auto_triggers.get(&ti)
    }

    pub fn get_manual_trigger_names(&self, ti: TriggerInfo) -> Option<&HashSet<TriggerName>> {
        self.manual_triggers.get(&ti)
    }

    pub fn set_response(&mut self, tn: TriggerName, tr: TriggerResponse) {
        self.responses.insert(tn, tr);
    }

    pub fn get_response(&self, tn: TriggerName) -> Option<TriggerResponse> {
        self.responses.get(&tn).cloned()
    }

    pub fn has_manual_triggers(&self, ti: TriggerInfo) -> bool {
        self.manual_triggers.contains_key(&ti)
    }

    pub fn has_auto_triggers(&self, ti: TriggerInfo) -> bool {
        self.auto_triggers.contains_key(&ti)
    }

    pub fn has_triggers(&self, ti: TriggerInfo) -> bool {
        self.manual_triggers.contains_key(&ti) || self.auto_triggers.contains_key(&ti)
    }

    pub fn get_auto_responses(&self, ti: TriggerInfo) -> Vec<TriggerResponse> {
        let tn_hs = self.auto_triggers.get(&ti);
        if tn_hs.is_some() {
            let mut v = Vec::with_capacity(tn_hs.unwrap().len());
            for tn in tn_hs.unwrap() {
                let tr = self.responses.get(tn);
                if tr.is_some() {
                    v.push(tr.unwrap().clone());
                }
            }
            v
        } else {
            Vec::new()
        }
    }

    pub fn get_manual_responses(&self, ti: TriggerInfo) -> Vec<TriggerResponse> {
        let tn_hs = self.manual_triggers.get(&ti);
        if tn_hs.is_some() {
            let mut v = Vec::with_capacity(tn_hs.unwrap().len());
            for tn in tn_hs.unwrap() {
                let tr = self.responses.get(tn);
                if tr.is_some() {
                    v.push(tr.unwrap().clone());
                }
            }
            v
        } else {
            Vec::new()
        }
    }
}
