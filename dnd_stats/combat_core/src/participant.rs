use std::collections::HashSet;
use std::fmt::Debug;

use crate::ability_scores::AbilityScores;
use crate::actions::ActionManager;
use crate::CCError;
use crate::conditions::ConditionManager;
use crate::damage::DamageType;
use crate::resources::ResourceManager;
use crate::skills::SkillManager;
use crate::triggers::TriggerManager;

pub trait Participant : Debug {
    fn get_ac(&self) -> isize;
    fn get_max_hp(&self) -> isize;
    fn get_prof(&self) -> isize;
    fn get_resistances(&self) -> &HashSet<DamageType>;
    fn get_ability_scores(&self) -> &AbilityScores;
    fn get_skill_manager(&self) -> &SkillManager;
    fn get_action_manager(&self) -> &ActionManager;
    fn get_resource_manager(&self) -> &ResourceManager;
    fn has_triggers(&self) -> bool;
    fn get_trigger_manager(&self) -> Option<&TriggerManager>;
    fn get_condition_manager(&self) -> &ConditionManager;

    fn register_pid(&mut self, pid: ParticipantId);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Team {
    Players,
    Enemies,
}

#[derive(Debug)]
pub struct TeamMember {
    pub team: Team,
    pub participant: Box<dyn Participant>,
}
impl TeamMember {
    pub fn new(team: Team, participant: Box<dyn Participant>) -> Self {
        TeamMember {
            team,
            participant,
        }
    }

    pub fn register_pid(&mut self, pid: ParticipantId) {
        self.participant.register_pid(pid);
    }
}

#[derive(Debug, Clone)]
pub struct ParticipantData {
    pub team: Team,
    pub ac: isize,
    pub max_hp: isize,
    pub resistances: HashSet<DamageType>,
}

impl From<&TeamMember> for ParticipantData {
    fn from(value: &TeamMember) -> Self {
        Self {
            team: value.team,
            ac: value.participant.get_ac(),
            max_hp: value.participant.get_max_hp(),
            resistances: value.participant.get_resistances().clone(),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ParticipantId(pub usize);
impl From<usize> for ParticipantId {
    fn from(value: usize) -> Self {
        ParticipantId(value)
    }
}

impl ParticipantId {
    // This is kind of a dirty hack, since Character's don't know their
    // ParticipantId until they are in an encounter. These will get
    // registered when the ParticipantManager is created...
    // For sure. Definitely.
    pub fn me() -> Self {
        Self(usize::MAX)
    }
}

#[derive(Debug)]
pub struct ParticipantManager { // In order of initiative
    participants: Vec<TeamMember>,
    initial_resources: Vec<ResourceManager>,
    initial_conditions: Vec<ConditionManager>,
    compiled: bool,
}

impl ParticipantManager {
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            initial_resources: Vec::new(),
            initial_conditions: Vec::new(),
            compiled: false,
        }
    }

    pub fn add_player(&mut self, player: Box<dyn Participant>) -> Result<(), CCError> {
        self.add_participant(TeamMember::new(Team::Players, player))
    }

    pub fn add_enemy(&mut self, enemy: Box<dyn Participant>) -> Result<(), CCError> {
        self.add_participant(TeamMember::new(Team::Enemies, enemy))
    }

    pub fn add_participant(&mut self, mut tm: TeamMember) -> Result<(), CCError> {
        if self.compiled {
            return Err(CCError::PMPushAfterCompile);
        }
        tm.register_pid(ParticipantId(self.len()));
        self.initial_resources.push(tm.participant.get_resource_manager().clone());
        self.initial_conditions.push(tm.participant.get_condition_manager().clone());
        self.participants.push(tm);
        Ok(())
    }

    pub fn compile(&mut self) {
        self.compiled = true;
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    pub fn len(&self) -> usize {
        self.participants.len()
    }

    pub fn get_participants(&self) -> &Vec<TeamMember> {
        &self.participants
    }

    pub fn get_participant(&self, pid: ParticipantId) -> &TeamMember {
        self.participants.get(pid.0).unwrap()
    }

    pub fn get_initial_rms(&self) -> Vec<ResourceManager> {
        self.initial_resources.clone()
    }

    pub fn get_initial_cms(&self) -> Vec<ConditionManager> {
        self.initial_conditions.clone()
    }
}
