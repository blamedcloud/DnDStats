use std::collections::{HashMap, HashSet};
use crate::attack::AttackResult;
use crate::damage::DamageTerm;
use crate::resources::ResourceName;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerType {
    WasHit,
    SuccessfulAttack,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerContext {
    AR(AttackResult),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriggerName {
    SneakAttack,
}

#[derive(Debug, Copy, Clone)]
pub enum TriggerAction {
    IncreaseAC(isize),
    AddAttackDamage(DamageTerm),
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
    pub triggers: HashMap<TriggerType, HashSet<TriggerName>>,
    pub responses: HashMap<TriggerName, TriggerResponse>,
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
            responses: HashMap::new(),
        }
    }

    pub fn add_trigger(&mut self, tt: TriggerType, tn: TriggerName) {
        if self.triggers.contains_key(&tt) {
            let mut oldv = self.triggers.remove(&tt).unwrap();
            oldv.insert(tn);
            self.triggers.insert(tt, oldv);
        } else {
            let mut hs = HashSet::new();
            hs.insert(tn);
            self.triggers.insert(tt, hs);
        }
    }

    pub fn add_triggers(&mut self, tt: TriggerType, tn_hs: HashSet<TriggerName>) {
        if self.triggers.contains_key(&tt) {
            let mut oldv = self.triggers.remove(&tt).unwrap();
            oldv.extend(tn_hs.into_iter());
            self.triggers.insert(tt, oldv);
        } else {
            self.triggers.insert(tt, tn_hs);
        }
    }

    pub fn replace_trigger(&mut self, tt: TriggerType, tn_hs: HashSet<TriggerName>) {
        self.triggers.insert(tt, tn_hs);
    }

    pub fn get_trigger_names(&self, tt: TriggerType) -> Option<&HashSet<TriggerName>> {
        self.triggers.get(&tt)
    }

    pub fn set_response(&mut self, tn: TriggerName, tr: TriggerResponse) {
        self.responses.insert(tn, tr);
    }

    pub fn get_response(&self, tn: TriggerName) -> Option<TriggerResponse> {
        self.responses.get(&tn).cloned()
    }

    pub fn has_triggers(&self, tt: TriggerType) -> bool {
        self.triggers.contains_key(&tt)
    }

    pub fn get_all_responses(&self, tt: TriggerType) -> Vec<TriggerResponse> {
        let tn_hs = self.triggers.get(&tt);
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
