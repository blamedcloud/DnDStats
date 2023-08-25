use std::cmp;
use std::collections::{HashMap, HashSet};
use crate::combat::{ActionName, ActionType};

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ResourceName {
    AT(ActionType),
    AN(ActionName),
}

impl From<ActionType> for ResourceName {
    fn from(value: ActionType) -> Self {
        ResourceName::AT(value)
    }
}
impl From<ActionName> for ResourceName {
    fn from(value: ActionName) -> Self {
        ResourceName::AN(value)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum RefreshTiming {
    StartRound,
    EndRound,
    StartMyTurn,
    EndMyTurn,
    StartOtherTurn,
    EndOtherTurn,
    ShortRest,
    LongRest,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RefreshBy {
    Const(usize),
    ToFull,
    ToEmpty,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ResourceCap {
    Soft(usize),
    Hard(usize),
}

impl ResourceCap {
    pub fn cap(&self) -> usize {
        match self {
            ResourceCap::Soft(c) => *c,
            ResourceCap::Hard(c) => *c,
        }
    }
}

#[derive(Clone)]
pub struct Resource {
    max: ResourceCap,
    current: usize,
    refresh_map: HashMap<RefreshTiming, RefreshBy>,
    expirations: HashSet<RefreshTiming>,
}

impl Resource {
    pub fn new(max: ResourceCap, current: usize) -> Self {
        Resource {
            max,
            current,
            refresh_map: HashMap::new(),
            expirations: HashSet::new(),
        }
    }

    pub fn get_max(&self) -> ResourceCap {
        self.max
    }
    pub fn set_max(&mut self, new_max: ResourceCap) {
        self.max = new_max;
    }

    pub fn get_current(&self) -> usize {
        self.current
    }

    pub fn spend(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }
    pub fn gain(&mut self, uses: usize) {
        if let ResourceCap::Hard(cap) = self.max {
            self.current = cmp::min(self.current + uses, cap);
        } else {
            self.current += uses;
        }
    }
    pub fn drain(&mut self) {
        self.current = 0;
    }

    pub fn add_refresh(&mut self, timing: RefreshTiming, by: RefreshBy) {
        self.expirations.remove(&timing);
        self.refresh_map.insert(timing, by);
    }

    pub fn add_expiration(&mut self, timing: RefreshTiming) {
        self.refresh_map.remove(&timing);
        self.expirations.insert(timing);
    }

    pub fn refresh(&mut self, timing: RefreshTiming) {
        if let Some(by) = self.refresh_map.get(&timing) {
            match by {
                RefreshBy::Const(c) => self.gain(*c),
                RefreshBy::ToFull => self.current = self.max.cap(),
                RefreshBy::ToEmpty => self.current = 0,
            }
        }
    }

    pub fn expires(&self, timing: RefreshTiming) -> bool {
        self.expirations.contains(&timing)
    }
}

impl From<ResourceCap> for Resource {
    fn from(value: ResourceCap) -> Self {
        Self {
            max: value,
            current: value.cap(),
            refresh_map: HashMap::new(),
            expirations: HashSet::new(),
        }
    }
}

#[derive(Clone)]
pub struct ResourceManager {
    perm_resources: HashMap<ResourceName, Resource>,
    temp_resources: HashMap<ResourceName, Resource>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            perm_resources: HashMap::new(),
            temp_resources: HashMap::new(),
        }
    }

    pub fn add_perm(&mut self, rn: ResourceName, res: Resource) {
        self.perm_resources.insert(rn, res);
    }

    pub fn add_temp(&mut self, rn: ResourceName, res: Resource) {
        if res.get_current() > 0 {
            self.temp_resources.insert(rn, res);
        }
    }

    pub fn has_resource(&self, rn: ResourceName) -> bool {
        self.perm_resources.contains_key(&rn) || self.temp_resources.contains_key(&rn)
    }

    pub fn get_current(&self, rn: ResourceName) -> usize {
        let temp_c = self.temp_resources.get(&rn).map(|r| r.get_current()).unwrap_or(0);
        let perm_c = self.perm_resources.get(&rn).map(|r| r.get_current()).unwrap_or(0);
        temp_c + perm_c
    }

    pub fn spend(&mut self, rn: ResourceName) {
        if let Some(res) = self.temp_resources.get_mut(&rn) {
            res.spend();
            if res.get_current() == 0 {
                self.temp_resources.remove(&rn);
            }
        } else {
            if let Some(res) = self.perm_resources.get_mut(&rn) {
                res.spend();
            }
        }
    }

    pub fn gain(&mut self, rn: ResourceName, uses: usize) {
        if let Some(res) = self.perm_resources.get_mut(&rn) {
            res.gain(uses);
        } else {
            if let Some(res) = self.temp_resources.get_mut(&rn) {
                res.gain(uses);
            }
        }
    }

    pub fn handle_timing(&mut self, rt: RefreshTiming) {
        self.handle_expirations(rt);
        self.handle_refreshes(rt);
    }

    fn handle_refreshes(&mut self, rt: RefreshTiming) {
        for (_, res) in self.perm_resources.iter_mut() {
            res.refresh(rt);
        }
    }

    fn handle_expirations(&mut self, rt: RefreshTiming) {
        let mut exp = HashSet::new();
        for (rn, res) in self.temp_resources.iter() {
            if res.expires(rt) {
                exp.insert(*rn);
            }
        }
        for rn in exp.into_iter() {
            self.temp_resources.remove(&rn);
        }
    }
}

pub fn create_resource_manager() -> ResourceManager {
    let mut rm = ResourceManager::new();
    let mut at_res = Resource::from(ResourceCap::Soft(1));
    at_res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToFull);

    rm.add_perm(ResourceName::AT(ActionType::Action), at_res.clone());
    rm.add_perm(ResourceName::AT(ActionType::BonusAction), at_res.clone());
    rm.add_perm(ResourceName::AT(ActionType::Reaction), at_res.clone());
    rm.add_perm(ResourceName::AT(ActionType::Movement), at_res);

    let mut sa_res = Resource::from(ResourceCap::Soft(0));
    sa_res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToEmpty);
    sa_res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToEmpty);
    rm.add_perm(ResourceName::AT(ActionType::SingleAttack), sa_res);

    rm
}
