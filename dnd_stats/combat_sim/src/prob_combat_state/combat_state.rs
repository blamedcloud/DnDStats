use std::collections::HashSet;
use character_builder::resources::ResourceManager;
use crate::participant::{ParticipantId, ParticipantManager};
use crate::prob_combat_state::combat_state::combat_log::combat_event::{CombatEvent, CombatTiming};
use crate::prob_combat_state::combat_state::combat_log::CombatLog;
use crate::prob_combat_state::combat_state::health::Health;
use crate::transposition::Transposition;

pub mod combat_log;
pub mod health;

#[derive(Debug, Clone)]
pub struct CombatState {
    logs: CombatLog,
    resources: ParticipantResources,
    health: ParticipantHealth,
    deaths: HashSet<ParticipantId>,
    last_combat_timing: Option<CombatTiming>,
}

type ParticipantResources = Vec<ResourceManager>;
type ParticipantHealth = Vec<Health>;

impl CombatState {
    pub fn new(pm: &ParticipantManager) -> Self {
        let resources = pm.get_initial_rms();
        let health = vec![Health::Healthy; pm.len()];
        Self {
            logs: CombatLog::new(),
            resources,
            health,
            deaths: HashSet::new(),
            last_combat_timing: None,
        }
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

    pub fn get_rm_mut(&mut self, pid: ParticipantId) -> &mut ResourceManager {
        self.resources.get_mut(pid.0).unwrap()
    }

    pub fn get_health(&self, pid: ParticipantId) -> Health {
        *self.health.get(pid.0).unwrap()
    }

    pub fn set_health(&mut self, pid: ParticipantId, h: Health) {
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

impl Transposition for CombatState {
    fn is_transposition(&self, other: &Self) -> bool {
        if self.logs.is_transposition(&other.logs) {
            if self.resources == other.resources {
                if self.health == other.health {
                    if self.deaths == other.deaths {
                        if self.last_combat_timing == other.last_combat_timing {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn merge_left(&mut self, other: Self) {
        self.logs.merge_left(other.logs);
    }
}
