use std::cmp;
use std::collections::{HashMap, HashSet};

use resource_amounts::{RefreshBy, ResourceCap};

use crate::actions::{ActionName, ActionType};
use crate::movement::Feet;
use crate::resources::resource_amounts::ResourceCount;
use crate::spells::SpellSlot;
use crate::triggers::TriggerName;

pub mod resource_amounts;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ResourceActionType {
    Action,
    SingleAttack,
    BonusAction,
    Reaction,
    FreeAction,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ResourceName {
    RAT(ResourceActionType),
    Movement,
    AN(ActionName),
    TN(TriggerName),
    SS(SpellSlot),
}

impl From<ActionType> for ResourceName {
    fn from(value: ActionType) -> Self {
        match value {
            ActionType::Action => ResourceName::RAT(ResourceActionType::Action),
            ActionType::SingleAttack => ResourceName::RAT(ResourceActionType::SingleAttack),
            ActionType::BonusAction => ResourceName::RAT(ResourceActionType::BonusAction),
            ActionType::Reaction => ResourceName::RAT(ResourceActionType::Reaction),
            ActionType::FreeAction => ResourceName::RAT(ResourceActionType::FreeAction),
            ActionType::HalfMove | ActionType::Movement => ResourceName::Movement,
        }
    }
}
impl From<ActionName> for ResourceName {
    fn from(value: ActionName) -> Self {
        ResourceName::AN(value)
    }
}
impl From<TriggerName> for ResourceName {
    fn from(value: TriggerName) -> Self {
        ResourceName::TN(value)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Resource {
    max: ResourceCap,
    current: ResourceCount,
    refresh_map: HashMap<RefreshTiming, RefreshBy>,
    expirations: HashSet<RefreshTiming>,
}

impl Resource {
    pub fn new(max: ResourceCap, current: ResourceCount) -> Self {
        Self {
            max,
            current,
            refresh_map: HashMap::new(),
            expirations: HashSet::new(),
        }
    }

    pub fn refill_lr(max: ResourceCap) -> Self {
        let mut refresh_map = HashMap::new();
        refresh_map.insert(RefreshTiming::LongRest, RefreshBy::ToFull);
        let mut current = ResourceCount::UnCapped;
        if !max.is_uncapped() {
            current = ResourceCount::Count(max.cap().unwrap());
        }
        Self {
            max,
            current,
            refresh_map,
            expirations: HashSet::new(),
        }
    }

    pub fn get_max(&self) -> ResourceCap {
        self.max
    }
    pub fn set_max(&mut self, new_max: ResourceCap) {
        self.max = new_max;
    }

    pub fn get_current(&self) -> ResourceCount {
        self.current
    }

    pub fn spend_many(&mut self, amount: usize) -> usize {
        if self.current >= amount {
            self.current -= amount;
            0
        } else {
            let leftover = amount - self.current.count().unwrap();
            self.current.set_count(0);
            leftover
        }
    }
    pub fn spend(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }
    pub fn gain(&mut self, uses: usize) {
        if let ResourceCap::Hard(cap) = self.max {
            self.current.set_count(cmp::min(self.current.count().unwrap() + uses, cap));
        } else {
            self.current += uses;
        }
    }
    pub fn gain_to_full(&mut self) {
        if self.max.is_uncapped() {
            self.current = ResourceCount::UnCapped;
        } else {
            self.current.set_count(self.max.cap().unwrap());
        }
    }
    pub fn drain(&mut self) {
        self.current.set_count(0);
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
                RefreshBy::ToFull => self.current = self.max.into(),
                RefreshBy::ToEmpty => self.current.set_count(0),
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
            current: value.into(),
            refresh_map: HashMap::new(),
            expirations: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn just_action_types() -> Self {
        create_basic_rm(Feet(30))
    }

    pub fn add_perm(&mut self, rn: ResourceName, res: Resource) {
        self.perm_resources.insert(rn, res);
    }

    pub fn add_temp(&mut self, rn: ResourceName, res: Resource) {
        if res.get_current() > 0 {
            self.temp_resources.insert(rn, res);
        }
    }

    pub fn set_spell_slots(&mut self, slots: [usize;10]) {
        // cantrips are always uncapped
        self.add_perm(ResourceName::SS(SpellSlot::Cantrip), Resource::refill_lr(ResourceCap::UnCapped));
        self.add_perm(ResourceName::SS(SpellSlot::First), Resource::refill_lr(ResourceCap::Hard(slots[1])));
        self.add_perm(ResourceName::SS(SpellSlot::Second), Resource::refill_lr(ResourceCap::Hard(slots[2])));
        self.add_perm(ResourceName::SS(SpellSlot::Third), Resource::refill_lr(ResourceCap::Hard(slots[3])));
        self.add_perm(ResourceName::SS(SpellSlot::Fourth), Resource::refill_lr(ResourceCap::Hard(slots[4])));
        self.add_perm(ResourceName::SS(SpellSlot::Fifth), Resource::refill_lr(ResourceCap::Hard(slots[5])));
        self.add_perm(ResourceName::SS(SpellSlot::Sixth), Resource::refill_lr(ResourceCap::Hard(slots[6])));
        self.add_perm(ResourceName::SS(SpellSlot::Seventh), Resource::refill_lr(ResourceCap::Hard(slots[7])));
        self.add_perm(ResourceName::SS(SpellSlot::Eighth), Resource::refill_lr(ResourceCap::Hard(slots[8])));
        self.add_perm(ResourceName::SS(SpellSlot::Ninth), Resource::refill_lr(ResourceCap::Hard(slots[9])));
    }

    pub fn has_resource(&self, rn: ResourceName) -> bool {
        self.perm_resources.contains_key(&rn) || self.temp_resources.contains_key(&rn)
    }

    pub fn check_counts(&self, spending: &HashMap<ResourceName, usize>) -> bool {
        for (rn, count) in spending {
            if !self.has_resource(*rn) {
                return false;
            }
            if  self.get_current(*rn) < *count {
                return false;
            }
        }
        true
    }

    pub fn get_cap(&self, rn: ResourceName) -> ResourceCap {
        let temp_c = self.temp_resources.get(&rn).map(|r| r.get_max()).unwrap_or(ResourceCap::Hard(0));
        let perm_c = self.perm_resources.get(&rn).map(|r| r.get_max()).unwrap_or(ResourceCap::Hard(0));
        temp_c + perm_c
    }

    pub fn set_cap(&mut self, rn: &ResourceName, cap: ResourceCap) {
        if self.perm_resources.contains_key(rn) {
            let res = self.perm_resources.get_mut(rn).unwrap();
            res.set_max(cap);
            res.gain_to_full();
        }
    }

    pub fn get_current(&self, rn: ResourceName) -> ResourceCount {
        let temp_c = self.temp_resources.get(&rn).map(|r| r.get_current()).unwrap_or(ResourceCount::Count(0));
        let perm_c = self.perm_resources.get(&rn).map(|r| r.get_current()).unwrap_or(ResourceCount::Count(0));
        temp_c + perm_c
    }

    pub fn is_full(&self, rn: ResourceName) -> bool {
        self.get_current(rn) == self.get_cap(rn)
    }

    pub fn spend_many(&mut self, rn: ResourceName, amount: usize) {
        let mut leftover = amount;
        if let Some(res) = self.temp_resources.get_mut(&rn) {
            leftover = res.spend_many(amount);
        }
        if leftover > 0 {
            if let Some(res) = self.perm_resources.get_mut(&rn) {
                res.spend_many(leftover);
            }
        }
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

    pub fn drain(&mut self, rn: ResourceName) {
        if self.temp_resources.contains_key(&rn) {
            self.temp_resources.remove(&rn);
        }
        if let Some(res) = self.perm_resources.get_mut(&rn) {
            res.drain();
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

pub fn create_basic_rm(speed: Feet) -> ResourceManager {
    let mut rm = ResourceManager::new();
    let mut at_res = Resource::from(ResourceCap::Soft(1));
    at_res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToFull);
    rm.add_perm(ResourceName::RAT(ResourceActionType::Reaction), at_res.clone());

    // clearing the action resources at the end of the turn is a small
    // optimization to make state merging more likely to happen.
    at_res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToEmpty);
    rm.add_perm(ResourceName::RAT(ResourceActionType::Action), at_res.clone());
    rm.add_perm(ResourceName::RAT(ResourceActionType::BonusAction), at_res);

    let mut move_res = Resource::from(ResourceCap::Soft(speed.0.abs() as usize));
    move_res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToEmpty);
    move_res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToFull);
    rm.add_perm(ResourceName::Movement, move_res);

    let mut sa_res = Resource::from(ResourceCap::Soft(0));
    sa_res.add_refresh(RefreshTiming::StartMyTurn, RefreshBy::ToEmpty);
    sa_res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToEmpty);
    rm.add_perm(ResourceName::RAT(ResourceActionType::SingleAttack), sa_res);

    rm
}
