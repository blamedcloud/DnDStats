use std::collections::{BTreeSet, HashMap, HashSet};
use std::vec;

use crate::{CCError, D20RollType};
use crate::ability_scores::Ability;
use crate::actions::ActionType;
use crate::combat_event::CombatTiming;
use crate::damage::{DamageFeature, DamageTerm};
use crate::participant::ParticipantId;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ConditionName {
    Concentration,
    Invisible,
    Prone,
    PlanarWarriorTarget,
}

impl ConditionName {
    pub fn get_basic_cond(&self) -> Result<Condition, CCError> {
        match self {
            ConditionName::Prone => {
                let effects = vec!(
                    ConditionEffect::AttackerMod(AttackDistance::Any, D20RollType::Disadvantage),
                    ConditionEffect::AtkTargetedMod(AttackDistance::Within5Ft, D20RollType::Advantage),
                    ConditionEffect::AtkTargetedMod(AttackDistance::Beyond5Ft, D20RollType::Disadvantage),
                );
                Ok(Condition {
                    effects,
                    lifetimes: vec!(ConditionLifetime::UntilSpendAT(ActionType::HalfMove))
                })
            }
            _ => Err(CCError::UnknownCondition)
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum RollAction {
    Saves,
    Skills,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum AttackDistance {
    Within5Ft,
    Beyond5Ft,
    Any
}

impl AttackDistance {
    pub fn applies_to(&self, dist: &AttackDistance) -> bool {
        self == &AttackDistance::Any || dist == &AttackDistance::Any || self == dist
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ConditionEffect {
    RollActionMod(RollAction, Ability, D20RollType), // ~ "you have dis.adv. on DEX saves
    AttackerMod(AttackDistance, D20RollType), // ~ "your attacks have advantage"
    AtkTargetedMod(AttackDistance, D20RollType), // ~ "attacks against you have advantage"
    TakeBonusDmgFrom(DamageTerm, ParticipantId), // planar warrior / hunter's mark
    TakeDmgFeatureFrom(DamageFeature, ParticipantId), // planar warrior convert to force dmg
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ConditionLifetime {
    Permanent,
    UntilSpendAT(ActionType),
    OnHitByAtk(ParticipantId),
    UntilTime(CombatTiming),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub effects: Vec<ConditionEffect>,
    // conditions go away when any of the lifetimes are met (logical OR)
    pub lifetimes: Vec<ConditionLifetime>,
}

impl Condition {
    pub fn register_pid(&mut self, pid: ParticipantId) {
        for ce in self.effects.iter_mut() {
            match ce {
                ConditionEffect::TakeBonusDmgFrom(_, old_pid) => {
                    if old_pid == &ParticipantId::me() {
                        old_pid.0 = pid.0;
                    }
                },
                ConditionEffect::TakeDmgFeatureFrom(_, old_pid) => {
                    if old_pid == &ParticipantId::me() {
                        old_pid.0 = pid.0;
                    }
                },
                _ => {}
            }
        }
        for cl in self.lifetimes.iter_mut() {
            match cl {
                ConditionLifetime::OnHitByAtk(old_pid) => {
                    if old_pid == &ParticipantId::me() {
                        old_pid.0 = pid.0;
                    }
                },
                ConditionLifetime::UntilTime(ct) => ct.register_pid(pid),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionManager {
    conditions: HashMap<ConditionName, Condition>,
    by_lifetime: HashMap<ConditionLifetime, BTreeSet<ConditionName>>,
}

impl ConditionManager {
    pub fn new() -> Self {
        Self {
            conditions: HashMap::new(),
            by_lifetime: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.conditions.len()
    }

    pub fn has_condition(&self, cond: &ConditionName) -> bool {
        self.conditions.contains_key(cond)
    }

    pub fn get_condition(&self, cn: &ConditionName) -> &Condition {
        self.conditions.get(cn).unwrap()
    }

    pub fn has_lifetime(&self, cl: &ConditionLifetime) -> bool {
        self.by_lifetime.contains_key(cl)
    }

    pub fn get_cns_for_lifetime(&self, cl: &ConditionLifetime) -> &BTreeSet<ConditionName> {
        self.by_lifetime.get(cl).unwrap()
    }

    pub fn add_basic_condition(&mut self, cn: ConditionName) -> Result<(), CCError> {
        self.add_condition(cn, cn.get_basic_cond()?);
        Ok(())
    }

    pub fn add_condition(&mut self, cn: ConditionName, cond: Condition) {
        let lifetimes = &cond.lifetimes;
        for lifetime in lifetimes {
            if self.by_lifetime.contains_key(lifetime) {
                let mut cns = self.by_lifetime.remove(lifetime).unwrap();
                cns.insert(cn);
                self.by_lifetime.insert(*lifetime, cns);
            } else {
                let mut cns = BTreeSet::new();
                cns.insert(cn);
                self.by_lifetime.insert(*lifetime, cns);
            }
        }
        self.conditions.insert(cn, cond);
    }

    pub fn remove_condition_by_name(&mut self, cn: &ConditionName) {
        let cond = self.conditions.remove(cn).unwrap();
        let lts = cond.lifetimes;
        for lt in lts {
            let o_cns = self.by_lifetime.remove(&lt);
            // this can be empty if we came here from remove_conditions_by_lifetime
            if o_cns.is_some() {
                let mut cns = o_cns.unwrap();
                cns.remove(cn);
                if cns.len() > 0 {
                    self.by_lifetime.insert(lt, cns);
                }
            }
        }
    }

    pub fn remove_conditions_by_lifetime(&mut self, lt: &ConditionLifetime) -> HashSet<ConditionName> {
        let mut removed_cns = HashSet::new();
        let cns = self.by_lifetime.remove(lt);
        if cns.is_some() {
            for cn in cns.unwrap() {
                self.remove_condition_by_name(&cn);
                removed_cns.insert(cn);
            }
        }
        removed_cns
    }

    pub fn get_atk_mod(&self, dist: AttackDistance) -> D20RollType {
        let mut atk_mod = D20RollType::Normal;
        for (_, cond) in &self.conditions {
            for effect in &cond.effects {
                if let ConditionEffect::AttackerMod(ad, roll) = effect {
                    if ad.applies_to(&dist) {
                        atk_mod += *roll;
                        if atk_mod == D20RollType::FixedNormal {
                            return atk_mod;
                        }
                    }
                }
            }
        }
        atk_mod
    }

    pub fn get_atk_target_mod(&self, dist: AttackDistance) -> D20RollType {
        let mut target_mod = D20RollType::Normal;
        for (_, cond) in &self.conditions {
            for effect in &cond.effects {
                if let ConditionEffect::AtkTargetedMod(ad, roll) = effect {
                    if ad.applies_to(&dist) {
                        target_mod += *roll;
                        if target_mod == D20RollType::FixedNormal {
                            return target_mod;
                        }
                    }
                }
            }
        }
        target_mod
    }

    pub fn overall_atk_mod(&self, target_cm: &ConditionManager, dist: AttackDistance) -> D20RollType {
        if dist == AttackDistance::Any {
            let melee = self.overall_atk_mod(target_cm, AttackDistance::Within5Ft);
            let ranged = self.overall_atk_mod(target_cm, AttackDistance::Beyond5Ft);
            melee.choose_better(&ranged)
        } else {
            let atk_mod = self.get_atk_mod(dist);
            let target_mod = target_cm.get_atk_target_mod(dist);
            atk_mod + target_mod
        }
    }

    pub fn overall_dmg_mods(&self, atker_pid: ParticipantId) -> (HashSet<DamageFeature>, Vec<DamageTerm>) {
        let mut dmg_feats = HashSet::new();
        let mut dmg_terms = Vec::new();
        for (_, cond) in self.conditions.iter() {
            for ce in cond.effects.iter() {
                match ce {
                    ConditionEffect::TakeBonusDmgFrom(term, pid) if pid == &atker_pid => {
                        dmg_terms.push(*term);
                    }
                    ConditionEffect::TakeDmgFeatureFrom(feat, pid) if pid == &atker_pid => {
                        dmg_feats.insert(*feat);
                    }
                    _ => {}
                }
            }
        }
        (dmg_feats, dmg_terms)
    }
}
