use std::collections::{HashMap, HashSet};

use crate::attack::AttackResult;
use crate::damage::DamageTerm;
use crate::resources::ResourceName;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerType {
    WasHit,
    SuccessfulAttack,
    OnKill,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerContext {
    NoContext,
    AR(AttackResult),
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
}

#[derive(Debug, Copy, Clone)]
pub enum TriggerAction {
    IncreaseAC(isize),
    AddAttackDamage(DamageTerm),
    AddResource(ResourceName, usize)
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
    pub triggers: HashMap<TriggerInfo, HashSet<TriggerName>>,
    pub responses: HashMap<TriggerName, TriggerResponse>,
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
            responses: HashMap::new(),
        }
    }

    pub fn add_trigger(&mut self, ti: TriggerInfo, tn: TriggerName) {
        if self.triggers.contains_key(&ti) {
            let mut oldv = self.triggers.remove(&ti).unwrap();
            oldv.insert(tn);
            self.triggers.insert(ti, oldv);
        } else {
            let mut hs = HashSet::new();
            hs.insert(tn);
            self.triggers.insert(ti, hs);
        }
    }

    pub fn add_triggers(&mut self, ti: TriggerInfo, tn_hs: HashSet<TriggerName>) {
        if self.triggers.contains_key(&ti) {
            let mut oldv = self.triggers.remove(&ti).unwrap();
            oldv.extend(tn_hs.into_iter());
            self.triggers.insert(ti, oldv);
        } else {
            self.triggers.insert(ti, tn_hs);
        }
    }

    pub fn replace_trigger(&mut self, ti: TriggerInfo, tn_hs: HashSet<TriggerName>) {
        self.triggers.insert(ti, tn_hs);
    }

    pub fn get_trigger_names(&self, ti: TriggerInfo) -> Option<&HashSet<TriggerName>> {
        self.triggers.get(&ti)
    }

    pub fn set_response(&mut self, tn: TriggerName, tr: TriggerResponse) {
        self.responses.insert(tn, tr);
    }

    pub fn get_response(&self, tn: TriggerName) -> Option<TriggerResponse> {
        self.responses.get(&tn).cloned()
    }

    pub fn has_triggers(&self, ti: TriggerInfo) -> bool {
        self.triggers.contains_key(&ti)
    }

    pub fn get_all_responses(&self, ti: TriggerInfo) -> Vec<TriggerResponse> {
        let tn_hs = self.triggers.get(&ti);
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
